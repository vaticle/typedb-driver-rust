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

use typedb_protocol::{
    Rule as RuleProto,
};
use typeql_lang::{parse_pattern, parse_variable};
use typeql_lang::pattern::{Pattern, Variable};
use crate::{common::Result, Error, error::InternalError, Rule};
use crate::error::ConnectionError;
use super::{FromProto, IntoProto, TryFromProto};

impl TryFromProto<RuleProto> for Rule {
    fn try_from_proto(proto: RuleProto) -> Result<Self> {
        let when = match parse_pattern(&proto.when) {
            Ok(Pattern::Conjunction(conjunction)) => conjunction,
            Ok(other) => return Err(Error::Other(format!("When parse error: {other:?}"))),
            Err(error) => return Err(Error::Other(format!("{error:?}"))),
        };
        let then = match parse_variable(&proto.then) {
            Ok(Variable::Thing(thing)) => thing,
            Ok(other) => return Err(Error::Other(format!("Then parse error: {other:?}"))),
            Err(error) => return Err(Error::Other(format!("{error:?}"))),
        };
        Ok(Self::new(proto.label, when, then))
    }
}

impl IntoProto<RuleProto> for Rule {
    fn into_proto(self) -> RuleProto {
        RuleProto {
            label: self.label,
            when: self.when.to_string(),
            then: self.then.to_string() ,
        }
    }
}

