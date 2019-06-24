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
