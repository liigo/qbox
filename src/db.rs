use std::default::Default;
use mysql::conn::MyOpts;
use mysql::conn::pool::MyPool;
use mysql::value::ToValue;
use mysql::error::MyResult;

pub fn have_a_try_to_mysql() {
    match connect_mysql() {
        Err(e) => {
            println!("mysql error: {}", e);
        }
        _ => {}
    }
}

fn connect_mysql() -> MyResult<()> {
    let opts = MyOpts {
        tcp_addr: Some("192.168.1.200".to_string()),
        tcp_port: 3306,
        user: Some("root".to_string()),
        pass: Some("".to_string()),
        db_name: Some("wrist_log".to_string()),
        ..Default::default()
    };
    let pool = try!(MyPool::new(opts));
    println!("created a mysql conn.");

    // 执行SQL查询
    let mut conn = try!(pool.get_conn());
    let mut query_result = try!(conn.query("select count(*) from logs"));
    if let Some(Ok(values)) = query_result.next() {
        println!("record count: {}", values[0].into_str());
    }

    // 执行SQL查询，利用for循环遍历记录集
    let mut conn = try!(pool.get_conn());
    let mut query_result = try!(conn.query("select * from logs"));
    for myresult in query_result {
        match myresult {
            Ok(ref values) => {
                println!("id:{}, time:{}, tag:{}",
                    values[0].into_str(),
                    values[1].into_str(),
                    values[4].into_str());
            }
            Err(e) => println!("{}", e),
        }
    }

    // 执行SQL语句，插入新记录
    let mut conn = try!(pool.get_conn());
    let mut query_result = try!(conn.query("insert into logs(time,tags) values(12345,'rust')"));
    println!("last_insert_id: {}", query_result.last_insert_id());
    println!("affected_rows: {}", query_result.affected_rows());

    Ok(())
}
