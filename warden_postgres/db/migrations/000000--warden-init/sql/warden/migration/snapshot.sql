create table warden.migration_snapshot (
  migration_id bigint not null primary key references warden.migration (id) on delete cascade on update cascade,
  "format" varchar(32) not null,
  data bytea
);

comment on table warden.migration_snapshot is 'Migration snapshots';
comment on column warden.migration_snapshot.format is 'Datum format (e.g. .tar.gz)';
comment on column warden.migration_snapshot.data is 'The snapshot data';
