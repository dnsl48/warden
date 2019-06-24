-- ---
-- version: 0.1
-- ...
-- timestamp: !!timestamp 2019-06-23T20:21:38.495+00:00
-- manifest:
--   - sql/warden.sql
--   - sql/warden/action_log.sql
--   - sql/warden/api/do_log.sql
--   - sql/warden/api/do_deploy_migration.sql
--   - sql/warden/api/do_register_migration.sql
--   - sql/warden/migration.sql
--   - sql/warden/api/get_latest_deployed_migration.sql
--   - sql/warden/do_base36_decode.sql
--   - sql/warden/do_base36_encode.sql
--   - sql/warden/migration/seal.sql
--   - sql/warden/migration/snapshot.sql
--   - sql/warden/migration/source.sql
--   - sql/warden/migration/triggers/fn__id_base36__populate.sql
--   - sql/warden/migration/triggers/tp__id_base36__populate.sql
-- ...

-- BEGIN: sql/warden.sql

create schema warden;
comment on schema warden is 'Warden data and functions';

-- END: sql/warden.sql

-- BEGIN: sql/warden/action_log.sql

create table warden.action_log (
  id bigserial not null primary key,
  ts timestamp with time zone not null default current_timestamp,
  "user" varchar(64) not null default session_user,
  client_address inet null default inet_client_addr(),
  session_pid int not null default pg_backend_pid(),
  the_action text
);

-- END: sql/warden/action_log.sql

-- BEGIN: sql/warden/api/do_log.sql

create or replace function
  warden.do_log(format_ text, variadic args text[])
  returns void
as $$
  insert into warden.action_log (the_action) values (format(format_, variadic args));
$$ language sql;

-- END: sql/warden/api/do_log.sql

-- BEGIN: sql/warden/api/do_deploy_migration.sql

-- ---
-- require:
--   - ./do_log.sql

create function
  warden.do_deploy_migration(
    id_ bigint
  )
returns void
as $$
declare
  migration_fullname_ text;
  deploy_ts_ timestamp with time zone;
  source_ text;
begin
  select
    lpad(m.id_base36, 6, '0') || '--' || m.name,
    m.deploy_ts,
    s.data
  into
    migration_fullname_,
    deploy_ts_,
    source_
  from
    warden.migration m
  left join
    warden.migration_source s
  on
    s.migration_id = m.id
  where
    m.id = id_;

  if deploy_ts_ is not null then
    raise exception 'Migration % has already been deployed', migration_fullname_;
  end if;

  if migration_fullname_ is null then
    raise exception 'Unknown migration with "%"', warden.do_base36_encode(id_);
  end if;

  execute source_;

  update warden.migration set deploy_ts = now() where id = id_;

  perform warden.do_log('Deployed | %s', migration_fullname_);

exception
  when others then
    perform warden.do_log('Deploy migration error: %s', SQLERRM);
    raise;
end;
$$ language plpgsql;

-- END: sql/warden/api/do_deploy_migration.sql

-- BEGIN: sql/warden/api/do_register_migration.sql

-- ---
-- require:
--   - ./do_log.sql

create function
  warden.do_register_migration(
    id_ bigint,
    name_ text,
    source_ text,
    snapshot_format_ varchar(32),
    snapshot_data_ bytea,
    seal_generated_at_ timestamp with time zone,
    seal_algo_ varchar(16),
    seal_data_ bytea
  )
returns void
as $$
declare
  migration_fullname_ text = id_::text || '--' || name_;
begin
  if id_ is null then
    raise exception 'Migration ID cannot be null';
  end if;

  if name_ is null then
    raise exception 'Migration Name cannot be null';
  end if;

  if source_ is null then
    raise exception 'Migration Source cannot be null';
  end if;

  if snapshot_format_ is null then
    raise exception 'Migration SnapshotFormat cannot be null';
  end if;

  if snapshot_data_ is null then
    raise exception 'Migration SnapshotData cannot be null';
  end if;

  insert into warden.migration (id, name) values (id_, name_);
  insert into warden.migration_source (migration_id, data) values (id_, source_);
  insert into warden.migration_snapshot (migration_id, format, data) values (id_, snapshot_format_, snapshot_data_);
  insert into warden.migration_seal (migration_id, generated_at, algo, value) values (id_, seal_generated_at_, seal_algo_, seal_data_);

  perform warden.do_log('Registered | %s', migration_fullname_);

exception
  when others then
    perform warden.do_log('Registering migration error: %s', SQLERRM);
    raise;
end;
$$ language plpgsql;

-- END: sql/warden/api/do_register_migration.sql

-- BEGIN: sql/warden/migration.sql

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

-- END: sql/warden/migration.sql

-- BEGIN: sql/warden/api/get_latest_deployed_migration.sql

-- ---
-- require:
--   - /warden/migration.sql

create function
  warden.get_latest_deployed_migration()
  returns bigint
as $$
  select id from warden.migration where deploy_ts is not null order by id desc limit 1;
$$ language sql;

-- END: sql/warden/api/get_latest_deployed_migration.sql

-- BEGIN: sql/warden/do_base36_decode.sql

create or replace function
  warden.do_base36_decode(value varchar)
  returns bigint
as $$
declare
  c smallint;
  i smallint;
  result bigint = 0;
begin
  for i in 1..char_length(value) loop
    c = ascii(substring(value from i for 1)::char);

    if c > ascii('9') then
      c = c - ascii('a') + 10;
    else
      c = c - ascii('0');
    end if;

    result = result * 36 + c;
  end loop;

  return result;
end;
$$ language 'plpgsql' immutable;

comment on function warden.do_base36_decode is 'Decodes base36 string into a big integer';

-- END: sql/warden/do_base36_decode.sql

-- BEGIN: sql/warden/do_base36_encode.sql

create or replace function
  warden.do_base36_encode(value bigint)
  returns varchar
as $$
declare
  result varchar = '';
  key smallint = 0;
begin
  if value < 0 then
    value = value * -1;
  end if;

  while value != 0 loop
    key = value % 36;

    if key < 10 then
      result = result || chr(ascii('0') + key);
    else
      result = result || chr(ascii('a') + (key - 10));
    end if;

    value = value / 36;
  end loop;

  if result = '' then
    result = '0';
  end if;

  return reverse(result);
end;
$$ language 'plpgsql' immutable;

comment on function warden.do_base36_encode is 'Encodes integer into base36 string';

-- END: sql/warden/do_base36_encode.sql

-- BEGIN: sql/warden/migration/seal.sql

create table warden.migration_seal (
  migration_id bigint not null primary key references warden.migration (id) on delete cascade on update cascade,
  generated_at timestamp with time zone not null,
  algo varchar(16) not null,
  value bytea
);

comment on table warden.migration_seal is 'Migration seal (signature)';
comment on column warden.migration_seal.generated_at is 'The seal generation timestamp';
comment on column warden.migration_seal.algo is 'Seal calculation algorithm';
comment on column warden.migration_seal.value is 'Seal value. Signature generated according to the algorithm';

-- END: sql/warden/migration/seal.sql

-- BEGIN: sql/warden/migration/snapshot.sql

create table warden.migration_snapshot (
  migration_id bigint not null primary key references warden.migration (id) on delete cascade on update cascade,
  "format" varchar(32) not null,
  data bytea
);

comment on table warden.migration_snapshot is 'Migration snapshots';
comment on column warden.migration_snapshot.format is 'Datum format (e.g. .tar.gz)';
comment on column warden.migration_snapshot.data is 'The snapshot data';

-- END: sql/warden/migration/snapshot.sql

-- BEGIN: sql/warden/migration/source.sql

create table warden.migration_source (
  migration_id bigint not null primary key references warden.migration (id) on delete cascade on update cascade,
  data text
);

comment on table warden.migration_source is 'Migration source (SQL)';
comment on column warden.migration_source.data is 'The actual migration SQL to be executed';

-- END: sql/warden/migration/source.sql

-- BEGIN: sql/warden/migration/triggers/fn__id_base36__populate.sql

create function
  warden.tf__migration__id_base36__populate ()
returns
  trigger
as $$
begin
  NEW.id_base36 = warden.do_base36_encode(NEW.id);
  return NEW;
end;
$$ language plpgsql;

-- END: sql/warden/migration/triggers/fn__id_base36__populate.sql

-- BEGIN: sql/warden/migration/triggers/tp__id_base36__populate.sql

create trigger
  "migration__id_base36__populate"
before
  insert or update of id
on
  warden.migration
for
  each row
execute procedure
  warden.tf__migration__id_base36__populate();

-- END: sql/warden/migration/triggers/tp__id_base36__populate.sql