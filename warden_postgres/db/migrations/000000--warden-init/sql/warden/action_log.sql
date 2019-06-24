 create table warden.action_log (
  id bigserial not null primary key,
  ts timestamp with time zone not null default current_timestamp,
  "user" varchar(64) not null default session_user,
  client_address inet null default inet_client_addr(),
  session_pid int not null default pg_backend_pid(),
  the_action text
);
