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
