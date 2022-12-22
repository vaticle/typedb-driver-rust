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

pub(crate) mod core {
    pub(crate) mod database_manager {
        use typedb_protocol::core_database_manager::{all, contains, create};

        pub(crate) fn contains_req(name: &str) -> contains::Req {
            contains::Req { name: name.to_owned() }
        }

        pub(crate) fn create_req(name: &str) -> create::Req {
            create::Req { name: name.to_owned() }
        }

        pub(crate) fn all_req() -> all::Req {
            all::Req {}
        }
    }

    pub(crate) mod database {
        use typedb_protocol::core_database::{delete, rule_schema, schema, type_schema};

        pub(crate) fn delete_req(name: &str) -> delete::Req {
            delete::Req { name: name.to_owned() }
        }

        pub(crate) fn rule_schema_req(name: &str) -> rule_schema::Req {
            rule_schema::Req { name: name.to_owned() }
        }

        pub(crate) fn schema_req(name: &str) -> schema::Req {
            schema::Req { name: name.to_owned() }
        }

        pub(crate) fn type_schema_req(name: &str) -> type_schema::Req {
            type_schema::Req { name: name.to_owned() }
        }
    }
}

pub(crate) mod cluster {
    pub(crate) mod server_manager {
        use typedb_protocol::server_manager::all;

        pub(crate) fn all_req() -> all::Req {
            all::Req {}
        }
    }

    pub(crate) mod user_manager {
        use typedb_protocol::cluster_user_manager::{all, contains, create};

        pub(crate) fn contains_req(username: &str) -> contains::Req {
            contains::Req { username: username.to_owned() }
        }

        pub(crate) fn create_req(username: &str, password: &str) -> create::Req {
            create::Req { username: username.to_owned(), password: password.to_owned() }
        }

        pub(crate) fn all_req() -> all::Req {
            all::Req {}
        }
    }

    pub(crate) mod user {
        use typedb_protocol::cluster_user::{delete, password, token};

        pub(crate) fn password_req(username: &str, password: &str) -> password::Req {
            password::Req { username: username.to_owned(), password: password.to_owned() }
        }

        pub(crate) fn token_req(username: &str) -> token::Req {
            token::Req { username: username.to_owned() }
        }

        pub(crate) fn delete_req(username: &str) -> delete::Req {
            delete::Req { username: username.to_owned() }
        }
    }

    pub(crate) mod database_manager {
        use typedb_protocol::cluster_database_manager::{all, get};

        pub(crate) fn get_req(name: &str) -> get::Req {
            get::Req { name: name.to_owned() }
        }

        pub(crate) fn all_req() -> all::Req {
            all::Req {}
        }
    }
}

pub(crate) mod session {
    use typedb_protocol::{
        session,
        session::{close, open, pulse},
        Options,
    };

    use crate::common::SessionID;

    pub(crate) fn close_req(session_id: SessionID) -> close::Req {
        close::Req { session_id: session_id.into() }
    }

    pub(crate) fn open_req(
        database: &str,
        session_type: session::Type,
        options: Options,
    ) -> open::Req {
        open::Req {
            database: database.to_owned(),
            r#type: session_type.into(),
            options: Some(options),
        }
    }

    pub(crate) fn pulse_req(session_id: SessionID) -> pulse::Req {
        pulse::Req { session_id: session_id.into() }
    }
}

pub(crate) mod transaction {
    use typedb_protocol::{
        transaction,
        transaction::{commit, open, rollback, stream},
        Options,
    };
    use uuid::Uuid;

    use crate::common::{RequestID, SessionID};

    pub(crate) fn client_msg(reqs: Vec<transaction::Req>) -> transaction::Client {
        transaction::Client { reqs }
    }

    pub(crate) fn stream_req(req_id: RequestID) -> transaction::Req {
        req_with_id(transaction::req::Req::StreamReq(stream::Req {}), req_id.into())
    }

    pub(crate) fn open_req(
        session_id: SessionID,
        transaction_type: transaction::Type,
        options: Options,
        network_latency_millis: i32,
    ) -> transaction::Req {
        req(transaction::req::Req::OpenReq(open::Req {
            session_id: session_id.into(),
            r#type: transaction_type.into(),
            options: Some(options),
            network_latency_millis,
        }))
    }

    pub(crate) fn commit_req() -> transaction::Req {
        req(transaction::req::Req::CommitReq(commit::Req {}))
    }

    pub(crate) fn rollback_req() -> transaction::Req {
        req(transaction::req::Req::RollbackReq(rollback::Req {}))
    }

    pub(super) fn req(req: transaction::req::Req) -> transaction::Req {
        transaction::Req { req_id: new_req_id(), metadata: Default::default(), req: Some(req) }
    }

    pub(super) fn req_with_id(req: transaction::req::Req, req_id: Vec<u8>) -> transaction::Req {
        transaction::Req { req_id, metadata: Default::default(), req: Some(req) }
    }

    fn new_req_id() -> Vec<u8> {
        Uuid::new_v4().as_bytes().to_vec()
    }
}

#[allow(dead_code)]
pub(crate) mod query_manager {
    use typedb_protocol::{
        query_manager,
        query_manager::{
            define, delete, explain, insert, match_aggregate, match_group, match_group_aggregate,
            r#match, undefine, update,
        },
        transaction,
        transaction::req::Req::QueryManagerReq,
        Options,
    };

    fn query_manager_req(req: query_manager::Req) -> transaction::Req {
        super::transaction::req(QueryManagerReq(req))
    }

    pub(crate) fn define_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::DefineReq(define::Req { query: query.to_owned() })),
        })
    }

    pub(crate) fn undefine_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::UndefineReq(undefine::Req {
                query: query.to_owned(),
            })),
        })
    }

    pub(crate) fn match_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::MatchReq(r#match::Req { query: query.to_owned() })),
        })
    }

    pub(crate) fn match_aggregate_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::MatchAggregateReq(match_aggregate::Req {
                query: query.to_owned(),
            })),
        })
    }

    pub(crate) fn match_group_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::MatchGroupReq(match_group::Req {
                query: query.to_owned(),
            })),
        })
    }

    pub(crate) fn match_group_aggregate_req(
        query: &str,
        options: Option<Options>,
    ) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::MatchGroupAggregateReq(
                match_group_aggregate::Req { query: query.to_owned() },
            )),
        })
    }

    pub(crate) fn insert_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::InsertReq(insert::Req { query: query.to_owned() })),
        })
    }

    pub(crate) fn delete_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::DeleteReq(delete::Req { query: query.to_owned() })),
        })
    }

    pub(crate) fn update_req(query: &str, options: Option<Options>) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options,
            req: Some(query_manager::req::Req::UpdateReq(update::Req { query: query.to_owned() })),
        })
    }

    pub(crate) fn explain_req(id: i64) -> transaction::Req {
        query_manager_req(query_manager::Req {
            options: None,
            req: Some(query_manager::req::Req::ExplainReq(explain::Req { explainable_id: id })),
        })
    }
}

#[allow(dead_code)]
pub(crate) mod thing {
    use typedb_protocol::{
        attribute, thing, thing::req::Req::AttributeGetOwnersReq, transaction,
        transaction::req::Req::ThingReq,
    };

    fn thing_req(req: thing::Req) -> transaction::Req {
        super::transaction::req(ThingReq(req))
    }

    pub(crate) fn attribute_get_owners_req(iid: &[u8]) -> transaction::Req {
        thing_req(thing::Req {
            iid: iid.to_vec(),
            req: Some(AttributeGetOwnersReq(attribute::get_owners::Req { filter: None })),
        })
    }
}
