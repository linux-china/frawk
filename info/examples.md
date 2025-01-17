AWK Examples
===============

# Read environment variable

```
db_url = ENVIRON["DB_URL"]
# or by `getenv` function
user_namme = getenv("USER", "guest")
```

**Attention**: zawk reads `.env` file by default.

# check variable null or empty

```
if(name =="") {
   name = "guest"
}
# or by ternary operator
guest_name = (name =="" ? "geust" : name)
```

# awk file help support

Use `zawk init demo.awk` to create an AWK file, and code as following:

```
#!/usr/bin/env zawk -f

# @desc this is a demo awk
# @meta author linux_china
# @meta version 0.1.0
# @var email current user email
# @env DB_URL database url

BEGIN {
    print email, ENVIRON["DB_URL"]
}

```

Then execute `./demo.awk --help` to display help.
 
# Best practise

* Use `BEGIN` and `END` block to initialize and finalize
* Ignore some lines by `pattern { next }`
* Process lines by `pattern { action }`
* Make associative array quickly:
    - `array[$1] = $2`, 
    - `arr = record("{host:localhost,port:1234}")`, 
    - `arr = pairs("a=b,c=d")`

# Prometheus text to CSV

```shell
$ zawk dump --prometheus http://localhost:8081/actuator/prometheus
```

# Output as CSV

`zawk -o csv 'BEGIN { print 1, "first,seccond"}'`

# NCSA Common Log Format

Please refer [Common Log Format](https://en.wikipedia.org/wiki/Common_Log_Format).

Log text:

```
127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326
```

Awk script: `clf-log.awk`

```awk
#!/usr/bin/env zawk -f

BEGIN {
    OFS = ","
    print "host", "ident", "authuser", "date", "status", "bytes", "method", "protocol", "path"
}

{
    date = $4 $5
    status = $(NF - 1)
    bytes = $NF
    method = trim($6, "\"")
    protocol = trim($(NF - 2), "\"")
    request_path = join_fields(7, NF - 3, " ")
    print $1, $2, $3, $4, trim(date, "[]"), status, bytes, method, protocol, request_path
}
```

# Spring Boot log

Log text: `spring.log`

```
2024-04-08T13:32:46.674+08:00  INFO 16314 --- [spring-boot-demo] [           main] o.a.c.c.C.[Tomcat].[localhost].[/]       : Initializing Spring embedded WebApplicationContext
2024-04-08T13:32:46.674+08:00  INFO 16314 --- [spring-boot-demo] [           main] w.s.c.ServletWebServerApplicationContext : Root WebApplicationContext: initialization completed in 847 ms
2024-04-08T13:32:47.624+08:00  INFO 16314 --- [spring-boot-demo] [on(2)-127.0.0.1] o.a.c.c.C.[Tomcat].[localhost].[/]       : Initializing Spring DispatcherServlet 'dispatcherServlet'
2024-04-08T13:32:47.624+08:00  INFO 16314 --- [spring-boot-demo] [on(2)-127.0.0.1] o.s.web.servlet.DispatcherServlet        : Initializing Servlet 'dispatcherServlet'
```

Awk script: `spring-log.awk`

```awk
#!/usr/bin/env zawk -f

BEGIN {
    OFS = ","
    print "timestamp", "level", "app_name", "thread", "logger", "msg"
}

# skip stacktrace or empty line
$1 !~ /^202\d-\d{2}-\d{2}/ {
    next
}

$6 == "[" {
    msg = join_fields(10, NF, " ")
    print $1, $2, trim($5, "[]"), trim($7, "[]"), $8, escape_csv(msg)
}

$6 != "[" {
    msg = join_fields(9, NF, " ")
    print $1, $2, trim($5, "[]"),trim($6, "[]"), $7, escape_csv(msg)
}
```

Awk with DuckDB:

```shell
$ ./spring-log.awk spring.log | duckdb -c "SELECT * FROM read_csv('/dev/stdin') where thread ='main'"
```

# logback

`%d{HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n`

https://logback.qos.ch/manual/layouts.html

`%d{HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n`

# remove duplicate lines

- print unique lines: `zawk '!visited[$0]++' demo.txt`
- print duplicated lines: `zawk 'visited[$0]++' demo.txt`

# USV support

[USV](https://github.com/SixArm/usv) markup:

- `␟`: U+001F/U+241F Unit Separator.
- `␞`: U+001E/U+241E Record Separator.

```
BEGIN {
    RS="␞\n"
    FS="␟"
}
```
