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

use super::{DatabaseManager, Session};
use crate::common::{ClusterConnection, Credential, Result, SessionType};

#[derive(Clone, Debug)]
pub struct Client {
    databases: DatabaseManager,
    cluster_connection: ClusterConnection,
}

impl Client {
    pub async fn new<T: AsRef<str> + Sync>(
        init_addresses: &[T],
        credential: Credential,
    ) -> Result<Self> {
        let cluster_connection = ClusterConnection::from_init(init_addresses, credential)?;
        Ok(Self { databases: DatabaseManager::new(cluster_connection.clone()), cluster_connection })
    }

    pub fn force_close(self) {
        self.cluster_connection.force_close();
    }

    pub fn databases(&mut self) -> &mut DatabaseManager {
        &mut self.databases
    }

    pub async fn session(
        &mut self,
        database_name: String,
        session_type: SessionType,
    ) -> Result<Session> {
        Session::new(self.databases.get(database_name).await?, session_type, self.clone()).await
    }
}
