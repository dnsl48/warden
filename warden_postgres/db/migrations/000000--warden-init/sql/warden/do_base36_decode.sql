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
