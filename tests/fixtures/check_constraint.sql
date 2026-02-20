-- Check constraint with ANY/ARRAY pattern
CREATE TABLE address (
    id bigint NOT NULL,
    coin text NOT NULL,
    account text NOT NULL,
    wallet_address character varying(255) NOT NULL,
    is_allocated boolean DEFAULT false NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT address_account_check CHECK ((account = ANY (ARRAY['client'::text, 'deposit'::text, 'payment'::text, 'stored'::text]))),
    CONSTRAINT address_coin_check CHECK ((coin = ANY (ARRAY['btc'::text, 'bch'::text, 'eth'::text, 'xrp'::text, 'hyt'::text])))
);
