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
