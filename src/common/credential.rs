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

use std::path::{Path, PathBuf};

use futures::StreamExt;
use tonic::{
    transport::{Certificate, ClientTlsConfig},
    Request,
};

use crate::Result;

#[derive(Clone, Debug)]
pub struct Credential {
    username: String,
    password: String,
    is_tls_enabled: bool,
    tls_root_ca: Option<PathBuf>,
}

impl Credential {
    pub fn new_with_tls(username: &str, password: &str, tls_root_ca: Option<&Path>) -> Self {
        Credential {
            username: username.to_owned(),
            password: password.to_owned(),
            is_tls_enabled: true,
            tls_root_ca: tls_root_ca.map(Path::to_owned),
        }
    }

    pub fn new_without_tls(username: &str, password: &str) -> Self {
        Credential {
            username: username.to_owned(),
            password: password.to_owned(),
            is_tls_enabled: false,
            tls_root_ca: None,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn is_tls_enabled(&self) -> bool {
        self.is_tls_enabled
    }

    pub fn tls_config(&self) -> Result<ClientTlsConfig> {
        if self.tls_root_ca.is_some() {
            Ok(ClientTlsConfig::new().ca_certificate(Certificate::from_pem(
                std::fs::read_to_string(self.tls_root_ca.as_ref().unwrap())?,
            )))
        } else {
            Ok(ClientTlsConfig::new())
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CallCredentials {
    credential: Credential,
    token: Option<String>,
}

impl CallCredentials {
    pub(super) fn new(credential: Credential) -> Self {
        Self { credential, token: None }
    }

    pub(super) fn username(&self) -> &str {
        self.credential.username()
    }

    pub(super) fn password(&self) -> &str {
        self.credential.password()
    }

    pub(super) fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub(super) fn reset_token(&mut self) {
        self.token = None;
    }

    pub(super) fn inject(&self, mut request: Request<()>) -> Request<()> {
        request.metadata_mut().insert("username", self.credential.username().try_into().unwrap());
        match &self.token {
            Some(token) => request.metadata_mut().insert("token", token.try_into().unwrap()),
            None => request
                .metadata_mut()
                .insert("password", self.credential.password().try_into().unwrap()),
        };
        request
    }
}
