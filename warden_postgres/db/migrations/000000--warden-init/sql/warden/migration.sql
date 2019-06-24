create table warden.migration (
  id bigint not null primary key,
  id_base36 varchar(32) not null unique,
  name text,
  sync_ts timestamp with time zone not null default current_timestamp,
  deploy_ts timestamp with time zone null default null
);

comment on table warden.migration is 'Registered migrations';
comment on column warden.migration.id is 'Migration ID';
comment on column warden.migration.id_base36 is 'Migration ID Base36 representation';
comment on column warden.migration.name is 'Migration name';
comment on column warden.migration.sync_ts is 'Registration timestamp';
comment on column warden.migration.deploy_ts is 'Migration deployment timestamp (null if it hasn''t been deployed)';
