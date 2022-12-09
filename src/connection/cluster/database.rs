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

use std::{fmt, fmt::Debug, future::Future, sync::Arc, time::Duration};

use futures::future::try_join_all;
use tokio::time::sleep;

use crate::{
    common::{
        error::{ClientError},
        rpc,
        rpc::builder::{
            cluster::database_manager::{all_req, get_req},
            core::database_manager::{contains_req, create_req},
        },
        Address, Result,
    },
    connection::server,
};

#[derive(Clone)]
pub struct Replica {
    pub(crate) address: Address,
    database_name: String,
    is_primary: bool,
    term: i64,
    is_preferred: bool,
    database: server::Database,
}

impl Debug for Replica {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Replica")
            .field("address", &self.address)
            .field("database_name", &self.database_name)
            .field("is_primary", &self.is_primary)
            .field("term", &self.term)
            .field("is_preferred", &self.is_preferred)
            .finish()
    }
}

impl Replica {
    fn new(
        name: &str,
        metadata: typedb_protocol::cluster_database::Replica,
        rpc_client: rpc::Client,
    ) -> Replica {
        Self {
            address: metadata.address.parse().expect("Invalid URI received from the server"),
            database_name: name.to_owned(),
            is_primary: metadata.primary,
            term: metadata.term,
            is_preferred: metadata.preferred,
            database: server::Database::new(name, rpc_client),
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pub name: String,
    replicas: Vec<Replica>,
    rpc_cluster_manager: Arc<rpc::ClusterClientManager>,
}

impl Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("cluster::Database")
            .field("name", &self.name)
            .field("replicas", &self.replicas)
            .finish()
    }
}

impl Database {
    async fn new(
        proto: typedb_protocol::ClusterDatabase,
        rpc_cluster_manager: Arc<rpc::ClusterClientManager>,
    ) -> Result<Self> {
        let replicas = proto
            .replicas
            .into_iter()
            .map(|replica| {
                let rpc_client =
                    rpc_cluster_manager.get(&replica.address.parse().unwrap()).into_core();
                Replica::new(&proto.name, replica, rpc_client)
            })
            .collect();
        Ok(Self { name: proto.name, replicas, rpc_cluster_manager })
    }

    async fn refresh_replicas(&mut self) {
        let res = self.rpc_cluster_manager.get_any().databases_get(get_req(&self.name)).await;
        let proto_replicas = res.unwrap().database.unwrap().replicas;
        self.replicas = proto_replicas
            .into_iter()
            .map(|replica| {
                let rpc_client =
                    self.rpc_cluster_manager.get(&replica.address.parse().unwrap()).into_core();
                Replica::new(&self.name, replica, rpc_client)
            })
            .collect();
    }

    pub(crate) async fn primary_replica(&mut self) -> Option<Replica> {
        // FIXME REMOVE
        let mut retries_left = 10;
        while retries_left > 0 {
            self.refresh_replicas().await;
            if let Some(replica) =
                self.replicas.iter().filter(|r| r.is_primary).max_by_key(|r| r.term).cloned()
            {
                return Some(replica);
            }
            println!("Retrying...");
            let _ = sleep(Duration::from_secs(2)).await;
            retries_left -= 1;
        }
        None
    }

    pub async fn delete(mut self) -> Result {
        self.primary_replica().await.unwrap().database.delete().await
    }

    pub async fn rule_schema(&mut self) -> Result<String> {
        self.primary_replica().await.unwrap().database.rule_schema().await
    }

    pub async fn schema(&mut self) -> Result<String> {
        self.primary_replica().await.unwrap().database.schema().await
    }

    pub async fn type_schema(&mut self) -> Result<String> {
        self.primary_replica().await.unwrap().database.type_schema().await
    }
}

#[derive(Clone, Debug)]
pub struct DatabaseManager {
    rpc_cluster_manager: Arc<rpc::ClusterClientManager>,
}

impl DatabaseManager {
    pub(crate) async fn new(rpc_cluster_manager: Arc<rpc::ClusterClientManager>) -> Result<Self> {
        Ok(Self { rpc_cluster_manager })
    }

    pub async fn get(&mut self, name: &str) -> Result<Database> {
        let maybe_proto_db = self
            .run_failsafe(move |mut server_client| {
                let req = get_req(name);
                async move { server_client.databases_get(req).await }
            })
            .await?
            .database;
        if let Some(proto_db) = maybe_proto_db {
            Database::new(proto_db, self.rpc_cluster_manager.clone()).await
        } else {
            Err(ClientError::DatabaseDoesNotExist(name.to_string()))?
        }
    }

    pub async fn contains(&mut self, name: &str) -> Result<bool> {
        Ok(self
            .run_failsafe(move |mut server_client| {
                let req = contains_req(name);
                async move { server_client.databases_contains(req).await }
            })
            .await?
            .contains)
    }

    pub async fn create(&mut self, name: &str) -> Result {
        self.run_failsafe(|mut server_client| {
            let req = create_req(name);
            async move { server_client.databases_create(req).await }
        })
        .await?;
        Ok(())
    }

    pub async fn all(&mut self) -> Result<Vec<Database>> {
        try_join_all(
            self.run_failsafe(|mut server_client| async move {
                server_client.databases_all(all_req()).await
            })
            .await?
            .databases
            .into_iter()
            .map(|proto_db| Database::new(proto_db, self.rpc_cluster_manager.clone())),
        )
        .await
    }

    async fn run_failsafe<F, P, R>(&mut self, task: F) -> Result<R>
    where
        F: Fn(rpc::ClusterClient) -> P,
        P: Future<Output = Result<R>>,
    {
        for client in self.rpc_cluster_manager.iter() {
            match task(client).await {
                // FIXME proper error handling
                Ok(r) => return Ok(r),
                Err(_) => (),
            }
        }
        Err(ClientError::UnableToConnect())?
    }
}
