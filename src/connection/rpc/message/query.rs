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

use typedb_protocol::query_manager;

use crate::{
    answer::{ConceptMap, Numeric},
    common::Result,
    Options,
};

#[derive(Debug)]
pub(crate) enum QueryRequest {
    // TODO options
    Define { query: String, options: Options },
    Undefine { query: String, options: Options },
    Delete { query: String, options: Options },

    Match { query: String, options: Options },
    Insert { query: String, options: Options },
    Update { query: String, options: Options },

    MatchAggregate { query: String, options: Options },

    Explain { explainable_id: i64, options: Options }, // FIXME ID type

    MatchGroup { query: String, options: Options },
    MatchGroupAggregate { query: String, options: Options },
}

#[derive(Debug)]
pub(crate) enum QueryResponse {
    // TODO options
    Define,
    Undefine,
    Delete,

    Match { answers: Vec<ConceptMap> },
    Insert { answers: Vec<ConceptMap> },
    Update { answers: Vec<ConceptMap> },

    MatchAggregate { answer: Numeric },

    Explain {}, // FIXME explanations

    MatchGroup {},          // FIXME ConceptMapGroup
    MatchGroupAggregate {}, // FIXME NumericGroup
}

impl From<QueryRequest> for query_manager::Req {
    fn from(value: QueryRequest) -> Self {
        let (req, options) = match value {
            QueryRequest::Define { query, options } => {
                (query_manager::req::Req::DefineReq(query_manager::define::Req { query }), options)
            }
            QueryRequest::Undefine { query, options } => (
                query_manager::req::Req::UndefineReq(query_manager::undefine::Req { query }),
                options,
            ),
            QueryRequest::Delete { query, options } => {
                (query_manager::req::Req::DeleteReq(query_manager::delete::Req { query }), options)
            }

            QueryRequest::Match { query, options } => {
                (query_manager::req::Req::MatchReq(query_manager::r#match::Req { query }), options)
            }
            QueryRequest::Insert { query, options } => {
                (query_manager::req::Req::InsertReq(query_manager::insert::Req { query }), options)
            }
            QueryRequest::Update { query, options } => {
                (query_manager::req::Req::UpdateReq(query_manager::update::Req { query }), options)
            }

            QueryRequest::MatchAggregate { query, options } => (
                query_manager::req::Req::MatchAggregateReq(query_manager::match_aggregate::Req {
                    query,
                }),
                options,
            ),

            _ => todo!(),
        };
        query_manager::Req { req: Some(req), options: Some(options.to_proto()) }
    }
}

impl From<query_manager::Res> for QueryResponse {
    fn from(value: query_manager::Res) -> Self {
        match value.res {
            Some(query_manager::res::Res::DefineRes(_)) => QueryResponse::Define,
            Some(query_manager::res::Res::UndefineRes(_)) => QueryResponse::Undefine,
            Some(query_manager::res::Res::DeleteRes(_)) => QueryResponse::Delete,
            Some(query_manager::res::Res::MatchAggregateRes(res)) => {
                QueryResponse::MatchAggregate { answer: res.answer.unwrap().try_into().unwrap() }
            }
            _ => todo!(),
        }
    }
}
impl From<query_manager::ResPart> for QueryResponse {
    fn from(value: query_manager::ResPart) -> Self {
        match value.res {
            Some(query_manager::res_part::Res::MatchResPart(res)) => QueryResponse::Match {
                answers: res
                    .answers
                    .into_iter()
                    .map(ConceptMap::from_proto)
                    .collect::<Result<_>>()
                    .unwrap(),
            },
            Some(query_manager::res_part::Res::InsertResPart(res)) => QueryResponse::Insert {
                answers: res
                    .answers
                    .into_iter()
                    .map(ConceptMap::from_proto)
                    .collect::<Result<_>>()
                    .unwrap(),
            },
            _ => todo!(),
        }
    }
}
