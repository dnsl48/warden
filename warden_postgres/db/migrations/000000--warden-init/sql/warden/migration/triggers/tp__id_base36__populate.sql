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
