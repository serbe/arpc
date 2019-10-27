CREATE TABLE IF NOT EXISTS 
    proxies (
        id        bigserial PRIMARY KEY,
        hostname  text NOT NULL,
        scheme    text NOT NULL,
        host      text NOT NULL,
        port      integer NOT NULL,
        work      boolean NOT NULL,
        anon      boolean NOT NULL,
        response  bigint NOT NULL,
        checks    integer NOT NULL DEFAULT 0,
        create_at TIMESTAMP without time zone,
        update_at TIMESTAMP without time zone DEFAULT now(),
        UNIQUE (hostname)
    );
