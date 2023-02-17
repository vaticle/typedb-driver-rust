/*
 * Copyright (C) 2022 Vaticle
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{Arc, Mutex},
    time::Duration,
};

use crossbeam::channel::bounded as bounded_blocking;
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel as unbounded_async, UnboundedReceiver, UnboundedSender},
        oneshot::channel as oneshot_async,
    },
    time::{sleep_until, Instant},
};

use super::{
    network::{
        channel::open_encrypted_channel,
        message::{DatabaseProto, Request, Response, TransactionRequest},
        stub::RPCStub,
        transmitter::{start_grpc_worker_encrypted, start_grpc_worker_plaintext},
    },
    TransactionStream,
};
use crate::{
    common::{
        error::{ClientError, Error},
        Address, Result, SessionID, SessionType, TransactionType,
    },
    connection::{runtime::BackgroundRuntime, OneShotSender},
    Credential, Options,
};

#[derive(Clone)]
pub struct Connection {
    server_connections: HashMap<Address, ServerConnection>,
    background_runtime: Arc<BackgroundRuntime>,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection").finish()
    }
}

async fn fetch_current_addresses(
    addresses: Vec<Address>,
    credential: Credential,
) -> Result<HashSet<Address>> {
    for address in addresses {
        let (channel, callcreds) = open_encrypted_channel(address.clone(), credential.clone())?;
        match RPCStub::new(address, channel, Some(callcreds)).await?.validated().await {
            Ok(mut client) => {
                return match client
                    .servers_all(Request::ServersAll.try_into()?)
                    .await?
                    .try_into()?
                {
                    Response::ServersAll { servers } => Ok(servers.into_iter().collect()),
                    _ => unreachable!(),
                }
            }
            Err(Error::Client(ClientError::UnableToConnect())) => (),
            Err(err) => Err(err)?,
        }
    }
    Err(ClientError::UnableToConnect())?
}

impl Connection {
    pub fn from_init<T: AsRef<str> + Sync>(
        init_addresses: &[T],
        credential: Credential,
    ) -> Result<Self> {
        let background_runtime = Arc::new(BackgroundRuntime::new()?);
        let init_addresses: Result<Vec<Address>> =
            init_addresses.iter().map(|addr| addr.as_ref().parse()).collect();
        let addresses = background_runtime
            .block_on(fetch_current_addresses(init_addresses?, credential.clone()))?;
        Self::new_encrypted(background_runtime, addresses, credential)
    }

    pub fn new_plaintext(address: impl AsRef<str>) -> Result<Self> {
        let address: Address = address.as_ref().parse()?;
        let background_runtime = Arc::new(BackgroundRuntime::new()?);
        let server_connection =
            ServerConnection::new_plaintext(background_runtime.clone(), address.clone())?;
        Ok(Self { server_connections: [(address, server_connection)].into(), background_runtime })
    }

    fn new_encrypted(
        background_runtime: Arc<BackgroundRuntime>,
        addresses: HashSet<Address>,
        credential: Credential,
    ) -> Result<Self> {
        let mut server_connections = HashMap::with_capacity(addresses.len());
        for address in addresses {
            let server_connection = ServerConnection::new_encrypted(
                background_runtime.clone(),
                address.clone(),
                credential.clone(),
            )?;
            server_connections.insert(address, server_connection);
        }
        Ok(Self { server_connections, background_runtime })
    }

    pub fn force_close(self) {
        self.server_connections.values().for_each(ServerConnection::force_close);
        self.background_runtime.force_close();
    }

    pub(crate) fn server_count(&self) -> usize {
        self.server_connections.len()
    }

    pub(crate) fn addresses(&self) -> impl Iterator<Item = &Address> {
        self.server_connections.keys()
    }

    pub(crate) fn get_server_connection(&self, address: &Address) -> ServerConnection {
        self.server_connections.get(address).cloned().unwrap()
    }

    pub(crate) fn iter_server_connections_cloned(
        &self,
    ) -> impl Iterator<Item = ServerConnection> + '_ {
        self.server_connections.values().cloned()
    }

    pub(crate) fn unable_to_connect(&self) -> Error {
        Error::Client(ClientError::ClusterUnableToConnect(
            self.addresses().map(Address::to_string).collect::<Vec<_>>().join(","),
        ))
    }
}

#[derive(Clone)]
pub(crate) struct ServerConnection {
    address: Address,
    background_runtime: Arc<BackgroundRuntime>,
    open_sessions: Arc<Mutex<HashMap<SessionID, UnboundedSender<()>>>>,
    request_sink: UnboundedSender<(Request, OneShotSender<Response>)>,
    shutdown_sink: UnboundedSender<()>,
}

impl fmt::Debug for ServerConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConnection").field("address", &self.address).finish()
    }
}

impl ServerConnection {
    fn new_plaintext(background_runtime: Arc<BackgroundRuntime>, address: Address) -> Result<Self> {
        let (request_sink, request_source) = unbounded_async();
        let (shutdown_sink, shutdown_source) = unbounded_async();
        background_runtime.spawn(start_grpc_worker_plaintext(
            address.clone(),
            request_source,
            shutdown_source,
        ));
        Ok(Self {
            address,
            background_runtime,
            open_sessions: Default::default(),
            request_sink,
            shutdown_sink,
        })
    }

    fn new_encrypted(
        background_runtime: Arc<BackgroundRuntime>,
        address: Address,
        credential: Credential,
    ) -> Result<Self> {
        let (shutdown_sink, shutdown_source) = unbounded_async();
        let (request_sink, request_source) = unbounded_async();
        background_runtime.spawn(start_grpc_worker_encrypted(
            address.clone(),
            credential,
            request_source,
            shutdown_source,
        ));
        Ok(Self {
            address,
            background_runtime,
            open_sessions: Default::default(),
            request_sink,
            shutdown_sink,
        })
    }

    pub(crate) fn address(&self) -> &Address {
        &self.address
    }

    async fn request_async(&self, request: Request) -> Result<Response> {
        if !self.background_runtime.is_open() {
            return Err(ClientError::ClientIsClosed().into());
        }
        let (response_sink, response) = oneshot_async();
        self.request_sink.send((request, OneShotSender::Async(response_sink)))?;
        response.await?
    }

    fn request_blocking(&self, request: Request) -> Result<Response> {
        if !self.background_runtime.is_open() {
            return Err(ClientError::ClientIsClosed().into());
        }
        let (response_sink, response) = bounded_blocking(0);
        self.request_sink.send((request, OneShotSender::Blocking(response_sink)))?;
        response.recv()?
    }

    pub(crate) fn force_close(&self) {
        let session_ids: Vec<SessionID> =
            self.open_sessions.lock().unwrap().keys().cloned().collect();
        for session_id in session_ids.into_iter() {
            self.close_session(session_id).ok();
        }
        self.shutdown_sink.send(()).ok();
    }

    pub(crate) async fn database_exists(&self, database_name: String) -> Result<bool> {
        match self.request_async(Request::DatabasesContains { database_name }).await? {
            Response::DatabasesContains { contains } => Ok(contains),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn create_database(&self, database_name: String) -> Result {
        self.request_async(Request::DatabaseCreate { database_name }).await?;
        Ok(())
    }

    pub(crate) async fn get_database_replicas(
        &self,
        database_name: String,
    ) -> Result<DatabaseProto> {
        match self.request_async(Request::DatabaseGet { database_name }).await? {
            Response::DatabaseGet { database } => Ok(database),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn all_databases(&self) -> Result<Vec<DatabaseProto>> {
        match self.request_async(Request::DatabasesAll).await? {
            Response::DatabasesAll { databases } => Ok(databases),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn database_schema(&self, database_name: String) -> Result<String> {
        match self.request_async(Request::DatabaseSchema { database_name }).await? {
            Response::DatabaseSchema { schema } => Ok(schema),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn database_type_schema(&self, database_name: String) -> Result<String> {
        match self.request_async(Request::DatabaseTypeSchema { database_name }).await? {
            Response::DatabaseTypeSchema { schema } => Ok(schema),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn database_rule_schema(&self, database_name: String) -> Result<String> {
        match self.request_async(Request::DatabaseRuleSchema { database_name }).await? {
            Response::DatabaseRuleSchema { schema } => Ok(schema),
            _ => unreachable!(),
        }
    }

    pub(crate) async fn delete_database(&self, database_name: String) -> Result {
        self.request_async(Request::DatabaseDelete { database_name }).await?;
        Ok(())
    }

    pub(crate) async fn open_session(
        &self,
        database_name: String,
        session_type: SessionType,
        options: Options,
    ) -> Result<(SessionID, Duration)> {
        match self
            .request_async(Request::SessionOpen { database_name, session_type, options })
            .await?
        {
            Response::SessionOpen { session_id, server_duration } => {
                let (shutdown_sink, shutdown_source) = unbounded_async();
                self.open_sessions.lock().unwrap().insert(session_id.clone(), shutdown_sink);
                self.background_runtime.spawn(session_pulse(
                    session_id.clone(),
                    self.request_sink.clone(),
                    shutdown_source,
                ));
                Ok((session_id, server_duration))
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn close_session(&self, session_id: SessionID) -> Result {
        if let Some(sink) = self.open_sessions.lock().unwrap().remove(&session_id) {
            sink.send(()).ok();
        }
        self.request_blocking(Request::SessionClose { session_id })?;
        Ok(())
    }

    pub(crate) async fn open_transaction(
        &self,
        session_id: SessionID,
        transaction_type: TransactionType,
        options: Options,
        network_latency: Duration,
    ) -> Result<TransactionStream> {
        let response = self
            .request_async(Request::Transaction(TransactionRequest::Open {
                session_id,
                transaction_type,
                options: options.clone(),
                network_latency,
            }))
            .await?;
        Ok(TransactionStream::new(&self.background_runtime, transaction_type, options, response))
    }
}

async fn session_pulse(
    session_id: SessionID,
    request_sink: UnboundedSender<(Request, OneShotSender<Response>)>,
    mut shutdown_source: UnboundedReceiver<()>,
) {
    const PULSE_INTERVAL: Duration = Duration::from_secs(5);
    let mut next_pulse = Instant::now();
    loop {
        select! {
            _ = sleep_until(next_pulse) => {
                let (response_sink, response) = oneshot_async();
                request_sink
                    .send((
                        Request::SessionPulse { session_id: session_id.clone() },
                        OneShotSender::Async(response_sink),
                    ))
                    .unwrap();
                response.await.unwrap().ok();
                next_pulse += PULSE_INTERVAL;
            }
            _ = shutdown_source.recv() => break,
        }
    }
}
