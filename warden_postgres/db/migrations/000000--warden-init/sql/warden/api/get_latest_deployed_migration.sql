-- ---
-- require:
--   - /warden/migration.sql

create function
  warden.get_latest_deployed_migration()
  returns bigint
as $$
  select id from warden.migration where deploy_ts is not null order by id desc limit 1;
$$ language sql;
