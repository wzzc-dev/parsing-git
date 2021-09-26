use sqlx::mysql::MySqlConnection;

#[derive(Debug, PartialEq, Eq, Clone, sqlx::FromRow)]
pub struct GitIndex { // 数据库中 git 索引对应的对象
    pub sha_1: Option<String>,
    pub obj_type: u8,
    pub size: u64,
    pub size_in_packfile: u64,
    pub offset_in_pack: u64,
    pub depth: u64,
    pub base_sha_1: Option<String>,
}
/**
 * 向数据库中插入索引
 */
pub async fn insert(git_index:&mut GitIndex, conn:&mut MySqlConnection) -> Result<(), sqlx::Error>{
    
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

/**
 * 向数据库中插入对象内容（现阶段全是以文本的形式）
 */
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
/**
 * 以一个 sha_1 值从数据库中得到一个索引对象
 */
pub async fn get_index(sha_1:&mut String,conn:&mut MySqlConnection)-> Result<GitIndex, sqlx::Error>{
    let sql = "select * from `git_index`
                    where sha_1 = ?";
    let recs = sqlx::query_as::<_, GitIndex>(sql)
        .bind(sha_1.to_string())
        .fetch_one(conn).await;

    
    recs
}