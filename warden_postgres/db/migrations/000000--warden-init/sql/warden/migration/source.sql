create table warden.migration_source (
  migration_id bigint not null primary key references warden.migration (id) on delete cascade on update cascade,
  data text
);

comment on table warden.migration_source is 'Migration source (SQL)';
comment on column warden.migration_source.data is 'The actual migration SQL to be executed';
