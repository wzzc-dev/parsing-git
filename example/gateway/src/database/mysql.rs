use sqlx::mysql::MySqlConnection;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GitIndex {
    pub sha_1: Option<String>,
    pub obj_type: u8,
    pub size: u64,
    pub size_in_packfile: u64,
    pub offset_in_pack: u64,
    pub depth: u64,
    pub base_sha_1: Option<String>,
}

pub async fn insert(git_index:&mut GitIndex, conn:&mut MySqlConnection) -> Result<(), sqlx::Error>{
    // let database_url = "mysql://root:123456@localhost:3306/git";

    // let pool = MySqlPoolOptions::new()
    //         .max_connections(5)
    //         .connect(database_url)
    //         .await?;
    // let mut conn = pool.acquire().await?;
    let sql = "insert into git_index (sha_1, obj_type, size, size_in_packfile, offset_in_pack, depth, base_sha_1) 
                    value (?, ?, ?, ?, ?, ?, ?)";
    sqlx::query(sql)
        .bind(git_index.sha_1.clone())
        .bind(git_index.obj_type)
        .bind(git_index.size)
        .bind(git_index.size_in_packfile)
        .bind(git_index.offset_in_pack)
        .bind(git_index.depth)
        .bind(git_index.base_sha_1.clone())
        .execute(conn)
    .await?;

    Ok(())

}
pub async fn insert_blob(sha_1:&mut String, context: String, conn:&mut MySqlConnection) -> Result<(), sqlx::Error>{
   
    let sql = "insert into `blob` (sha_1, context) 
                    values (?, ?)";
    sqlx::query(sql)
        .bind(sha_1.to_string())
        .bind(context)
        .execute(conn)
    .await?;

    Ok(())

}