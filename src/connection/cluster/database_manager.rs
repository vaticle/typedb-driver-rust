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

use std::future::Future;

use super::Database;
use crate::{
    common::{error::ClientError, ClusterConnection, ClusterServerConnection, Result},
    connection::server,
};

#[derive(Clone, Debug)]
pub struct DatabaseManager {
    cluster_connection: ClusterConnection,
}

impl DatabaseManager {
    pub(super) fn new(cluster_connection: ClusterConnection) -> Self {
        Self { cluster_connection }
    }

    pub async fn get(&mut self, name: String) -> Result<Database> {
        Database::get(name, self.cluster_connection.clone()).await
    }

    pub async fn contains(&mut self, name: String) -> Result<bool> {
        self.run_failsafe(name, move |database, server_connection, _| async move {
            server_connection.database_exists(database.name().to_owned()).await
        })
        .await
    }

    pub async fn create(&mut self, name: String) -> Result {
        self.run_failsafe(name, |database, server_connection, _| async move {
            server_connection.create_database(database.name().to_owned()).await
        })
        .await
    }

    pub async fn all(&mut self) -> Result<Vec<Database>> {
        let mut error_buffer = Vec::with_capacity(self.cluster_connection.server_count());
        for server_connection in self.cluster_connection.iter_server_connections_cloned() {
            match server_connection.all_databases().await {
                Ok(list) => {
                    return list
                        .into_iter()
                        .map(|proto_db| Database::new(proto_db, self.cluster_connection.clone()))
                        .collect()
                }
                Err(err) => {
                    error_buffer.push(format!("- {}: {}", server_connection.address(), err))
                }
            }
        }
        Err(ClientError::ClusterAllNodesFailed(error_buffer.join("\n")))?
    }

    async fn run_failsafe<F, P, R>(&mut self, name: String, task: F) -> Result<R>
    where
        F: Fn(server::Database, ClusterServerConnection, bool) -> P,
        P: Future<Output = Result<R>>,
    {
        Database::get(name, self.cluster_connection.clone()).await?.run_failsafe(&task).await
    }
}
