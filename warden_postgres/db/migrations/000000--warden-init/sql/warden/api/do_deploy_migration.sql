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
