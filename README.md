# arpc
Client
    to RpcServer
        join
        put urls
        get int
        get_anon int
    from RpcServer
        put urls

RpcServer
    to Client
        put urls
    to Manager
        put urls
        get_typed int
    from Client
        join
        put urls
        get int
        get_anon int
    from Manager
        put urls

Manager
    to RpcServer
        put urls
    to PgDb
        get_typed int
        put proxy
    to Worker
        put url
    from RpcServer
        put urls
        get_typed int
    from PgDb
        put urls
    from Worker
        put proxy

PgDb
    to Manager
        put urls
    from Manager
        get_typed int
        put proxy

Worker
    to Manager
        put proxy
    from Manager
        put url