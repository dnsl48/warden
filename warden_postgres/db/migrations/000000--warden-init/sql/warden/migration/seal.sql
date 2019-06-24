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
