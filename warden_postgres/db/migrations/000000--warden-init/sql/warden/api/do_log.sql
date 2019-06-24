create or replace function
  warden.do_log(format_ text, variadic args text[])
  returns void
as $$
  insert into warden.action_log (the_action) values (format(format_, variadic args));
$$ language sql;
