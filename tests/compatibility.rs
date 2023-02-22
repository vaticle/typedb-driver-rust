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

use std::path::PathBuf;

use futures::{StreamExt, TryFutureExt};
use serial_test::serial;
use typedb_client::{
    Connection, Credential, Database, DatabaseManager, Session,
    SessionType::{Data, Schema},
    TransactionType::Write,
};

const TEST_DATABASE: &str = "test";

#[test]
#[serial]
fn basic_async_std() {
    async_std::task::block_on(async {
        let connection = new_cluster_connection()?;
        create_test_database_with_schema(connection.clone(), "define person sub entity;").await?;
        let mut databases = DatabaseManager::new(connection);
        assert!(databases.contains(TEST_DATABASE.into()).await?);

        let session = Session::new(databases.get(TEST_DATABASE.into()).await?, Data).await?;
        let transaction = session.transaction(Write).await?;
        let answer_stream = transaction.query().match_("match $x sub thing;")?;
        let results: Vec<_> = answer_stream.collect().await;
        transaction.commit().await?;
        assert_eq!(results.len(), 5);
        assert!(results.into_iter().all(|res| res.is_ok()));
        Ok::<(), typedb_client::Error>(())
    })
    .unwrap();
}

#[test]
#[serial]
fn basic_smol() {
    smol::block_on(async {
        let connection = new_cluster_connection()?;
        create_test_database_with_schema(connection.clone(), "define person sub entity;").await?;
        let mut databases = DatabaseManager::new(connection);
        assert!(databases.contains(TEST_DATABASE.into()).await?);

        let session = Session::new(databases.get(TEST_DATABASE.into()).await?, Data).await?;
        let transaction = session.transaction(Write).await?;
        let answer_stream = transaction.query().match_("match $x sub thing;")?;
        let results: Vec<_> = answer_stream.collect().await;
        transaction.commit().await?;
        assert_eq!(results.len(), 5);
        assert!(results.into_iter().all(|res| res.is_ok()));
        Ok::<(), typedb_client::Error>(())
    })
    .unwrap();
}

#[test]
#[serial]
fn basic_futures() {
    futures::executor::block_on(async {
        let connection = new_cluster_connection()?;
        create_test_database_with_schema(connection.clone(), "define person sub entity;").await?;
        let mut databases = DatabaseManager::new(connection);
        assert!(databases.contains(TEST_DATABASE.into()).await?);

        let session = Session::new(databases.get(TEST_DATABASE.into()).await?, Data).await?;
        let transaction = session.transaction(Write).await?;
        let answer_stream = transaction.query().match_("match $x sub thing;")?;
        let results: Vec<_> = answer_stream.collect().await;
        transaction.commit().await?;
        assert_eq!(results.len(), 5);
        assert!(results.into_iter().all(|res| res.is_ok()));
        Ok::<(), typedb_client::Error>(())
    })
    .unwrap();
}

fn new_cluster_connection() -> typedb_client::Result<Connection> {
    Connection::new_encrypted(
        &["localhost:11729", "localhost:21729", "localhost:31729"],
        Credential::with_tls(
            "admin",
            "password",
            Some(&PathBuf::from(
                std::env::var("ROOT_CA")
                    .expect("ROOT_CA environment variable needs to be set for cluster tests to run"),
            )),
        ),
    )
}

async fn create_test_database_with_schema(connection: Connection, schema: &str) -> typedb_client::Result {
    let mut databases = DatabaseManager::new(connection);
    if databases.contains(TEST_DATABASE.into()).await? {
        databases.get(TEST_DATABASE.into()).and_then(Database::delete).await?;
    }
    databases.create(TEST_DATABASE.into()).await?;

    let database = databases.get(TEST_DATABASE.into()).await?;
    let session = Session::new(database, Schema).await?;
    let transaction = session.transaction(Write).await?;
    transaction.query().define(schema).await?;
    transaction.commit().await?;
    Ok(())
}
