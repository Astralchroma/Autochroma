CREATE TYPE MODULE AS ENUM ('nom');

CREATE TABLE modules (
	guild  BIGINT PRIMARY KEY, 
	module MODULE NOT NULL,

	UNIQUE(guild, module)
);
