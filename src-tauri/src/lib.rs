use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::Command, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

struct Db(Mutex<Connection>);
struct ProjectionStore(Mutex<Option<serde_json::Value>>);
type Result<T> = std::result::Result<T, String>;

#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct DashboardStats { songs: i64, sections: i64, bible_versions: i64, bible_verses: i64, media: i64 }
#[derive(Serialize, Deserialize, Clone)] #[serde(rename_all = "camelCase")]
struct TranslationInput { language: String, content: String }
#[derive(Serialize, Deserialize, Clone)] #[serde(rename_all = "camelCase")]
struct SectionInput { id: Option<i64>, section_type: String, label: String, content: String, position: i64, #[serde(default)] translations: Vec<TranslationInput> }
#[derive(Serialize, Deserialize)] #[serde(rename_all = "camelCase")]
struct SongInput { id: Option<i64>, title: String, author: String, category: String, language: String, copyright_info: String, media_id: Option<i64>, sections: Vec<SectionInput> }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct Song { id: i64, title: String, author: String, category: String, language: String, copyright_info: String, media_id: Option<i64>, sections: Vec<SectionInput> }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct BibleVersion { id: i64, name: String, abbreviation: String, language: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct BibleBook { book_number: i64, name: String, abbreviation: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct BibleVerse { id: i64, version_id: i64, version: String, language: String, book: String, book_number: i64, chapter: i64, verse: i64, text: String, reference: String }
#[derive(Deserialize)] struct BibleImport { name: String, abbreviation: String, language: String, books: Vec<BibleBookImport> }
#[derive(Deserialize)] struct BibleBookImport { name: String, abbreviation: String, number: i64, chapters: Vec<Vec<String>> }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
struct MediaFile { id: i64, name: String, file_name: String, path: String, media_type: String, is_default: bool }
#[derive(Serialize, Deserialize)] #[serde(rename_all = "camelCase")]
struct ProjectionSettings { font_family: String, font_size: i64, text_color: String, alignment: String, overlay_opacity: f64, transition_ms: i64, default_media_id: Option<i64> }

fn db_conn<'a>(state: &'a State<'_, Db>) -> std::sync::MutexGuard<'a, Connection> { state.0.lock().expect("database mutex poisoned") }
fn scalar(conn: &Connection, sql: &str) -> rusqlite::Result<i64> { conn.query_row(sql, [], |r| r.get(0)) }

#[tauri::command]
fn dashboard_stats(db: State<Db>) -> Result<DashboardStats> {
 let c=db_conn(&db); Ok(DashboardStats { songs:scalar(&c,"SELECT count(*) FROM songs").map_err(err)?, sections:scalar(&c,"SELECT count(*) FROM song_sections").map_err(err)?, bible_versions:scalar(&c,"SELECT count(*) FROM bible_versions").map_err(err)?, bible_verses:scalar(&c,"SELECT count(*) FROM bible_verses").map_err(err)?, media:scalar(&c,"SELECT count(*) FROM media_files").map_err(err)? })
}

#[tauri::command]
fn list_songs(query: Option<String>, db: State<Db>) -> Result<Vec<Song>> {
 let c=db_conn(&db); let needle=format!("%{}%", query.unwrap_or_default());
 let mut stmt=c.prepare("SELECT DISTINCT s.id,s.title,s.author,s.category,s.language,s.copyright_info,s.media_id FROM songs s LEFT JOIN song_sections x ON x.song_id=s.id WHERE s.title LIKE ?1 OR s.author LIKE ?1 OR s.category LIKE ?1 OR s.language LIKE ?1 OR x.content LIKE ?1 ORDER BY s.title").map_err(err)?;
 let rows=stmt.query_map([needle], |r| Ok((r.get::<_,i64>(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).map_err(err)?;
 let mut out=vec![]; for row in rows { let (id,title,author,category,language,copyright_info,media_id)=row.map_err(err)?; out.push(Song{id,title,author,category,language,copyright_info,media_id,sections:load_sections(&c,id)?}); } Ok(out)
}
fn load_sections(c:&Connection,song_id:i64)->Result<Vec<SectionInput>> { let mut s=c.prepare("SELECT id,section_type,label,content,position FROM song_sections WHERE song_id=? ORDER BY position,id").map_err(err)?; let rows=s.query_map([song_id],|r|Ok((r.get::<_,i64>(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(err)?;let mut out=vec![];for row in rows{let(id,section_type,label,content,position)=row.map_err(err)?;let mut ts=c.prepare("SELECT language,content FROM song_translations WHERE section_id=? ORDER BY language").map_err(err)?;let translations=ts.query_map([id],|r|Ok(TranslationInput{language:r.get(0)?,content:r.get(1)?})).map_err(err)?.map(|x|x.map_err(err)).collect::<Result<Vec<_>>>()?;out.push(SectionInput{id:Some(id),section_type,label,content,position,translations});}Ok(out) }

#[tauri::command]
fn save_song(song: SongInput, db: State<Db>) -> Result<i64> {
 let mut c=db_conn(&db); let tx=c.transaction().map_err(err)?;
 let id=if let Some(id)=song.id { tx.execute("UPDATE songs SET title=?,author=?,category=?,language=?,copyright_info=?,media_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?",params![song.title,song.author,song.category,song.language,song.copyright_info,song.media_id,id]).map_err(err)?; tx.execute("DELETE FROM song_sections WHERE song_id=?",[id]).map_err(err)?; id } else { tx.execute("INSERT INTO songs(title,author,category,language,copyright_info,media_id) VALUES(?,?,?,?,?,?)",params![song.title,song.author,song.category,song.language,song.copyright_info,song.media_id]).map_err(err)?; tx.last_insert_rowid() };
 for x in song.sections { tx.execute("INSERT INTO song_sections(song_id,section_type,label,content,position) VALUES(?,?,?,?,?)",params![id,x.section_type,x.label,x.content,x.position]).map_err(err)?;let section_id=tx.last_insert_rowid();for t in x.translations{tx.execute("INSERT INTO song_translations(song_id,section_id,language,content) VALUES(?,?,?,?)",params![id,section_id,t.language,t.content]).map_err(err)?;} } tx.commit().map_err(err)?; Ok(id)
}
#[tauri::command]
fn delete_song(id:i64,db:State<Db>)->Result<()> { db_conn(&db).execute("DELETE FROM songs WHERE id=?",[id]).map_err(err)?; Ok(()) }
#[tauri::command]
fn import_song_text(path:String,db:State<Db>)->Result<i64>{let content=fs::read_to_string(&path).map_err(err)?;let title=Path::new(&path).file_stem().and_then(|x|x.to_str()).unwrap_or("Imported song");let blocks=content.replace("\r\n","\n").split("\n\n").map(str::trim).filter(|x|!x.is_empty()).map(str::to_string).collect::<Vec<_>>();let c=db_conn(&db);c.execute("INSERT INTO songs(title,category,language) VALUES(?,'Imported','English')",[title]).map_err(err)?;let id=c.last_insert_rowid();for(i,block)in blocks.iter().enumerate(){let mut lines=block.lines();let first=lines.next().unwrap_or("");let (label,body)=if first.ends_with(':')&&(first.len()<40){(first.trim_end_matches(':').to_string(),lines.collect::<Vec<_>>().join("\n"))}else{(format!("Verse {}",i+1),block.clone())};let kind=label.split_whitespace().next().unwrap_or("verse").to_lowercase();c.execute("INSERT INTO song_sections(song_id,section_type,label,content,position) VALUES(?,?,?,?,?)",params![id,kind,label,body,i as i64]).map_err(err)?;}Ok(id)}
#[tauri::command]
fn export_songs(path:String,db:State<Db>)->Result<()>{let songs=list_songs(None,db)?;let json=serde_json::to_string_pretty(&songs).map_err(err)?;fs::write(path,json).map_err(err)}

#[tauri::command]
fn list_bible_versions(db:State<Db>)->Result<Vec<BibleVersion>> { let c=db_conn(&db); let mut s=c.prepare("SELECT id,name,abbreviation,language FROM bible_versions ORDER BY language,name").map_err(err)?; let out=s.query_map([],|r|Ok(BibleVersion{id:r.get(0)?,name:r.get(1)?,abbreviation:r.get(2)?,language:r.get(3)?})).map_err(err)?.map(|x|x.map_err(err)).collect();out }
#[tauri::command]
fn list_bible_books(version_id:i64,db:State<Db>)->Result<Vec<BibleBook>>{let c=db_conn(&db);let mut s=c.prepare("SELECT book_number,name,abbreviation FROM bible_books WHERE version_id=? ORDER BY book_number").map_err(err)?;let out=s.query_map([version_id],|r|Ok(BibleBook{book_number:r.get(0)?,name:r.get(1)?,abbreviation:r.get(2)?})).map_err(err)?.map(|x|x.map_err(err)).collect();out}
#[tauri::command]
fn bible_chapter_count(version_id:i64,book_number:i64,db:State<Db>)->Result<i64>{db_conn(&db).query_row("SELECT COALESCE(MAX(v.chapter),0) FROM bible_verses v JOIN bible_books b ON b.id=v.book_id WHERE v.version_id=? AND b.book_number=?",params![version_id,book_number],|r|r.get(0)).map_err(err)}
#[tauri::command]
fn bible_verse_count(version_id:i64,book_number:i64,chapter:i64,db:State<Db>)->Result<i64>{db_conn(&db).query_row("SELECT COALESCE(MAX(v.verse),0) FROM bible_verses v JOIN bible_books b ON b.id=v.book_id WHERE v.version_id=? AND b.book_number=? AND v.chapter=?",params![version_id,book_number,chapter],|r|r.get(0)).map_err(err)}

#[tauri::command]
fn search_bible(query:String, version_ids:Vec<i64>, db:State<Db>)->Result<Vec<BibleVerse>> {
 let c=db_conn(&db); let q=query.trim(); let mut book_term=q.to_string(); let mut chapter=None; let mut verse=None;
 if let Some((left,v))=q.rsplit_once(':') { verse=v.parse::<i64>().ok(); if let Some((b,ch))=left.rsplit_once(' ') { book_term=b.to_string(); chapter=ch.parse::<i64>().ok(); } }
 else if let Some((b,ch))=q.rsplit_once(' ') { if let Ok(n)=ch.parse::<i64>() { book_term=b.to_string(); chapter=Some(n); } }
 let ids=if version_ids.is_empty(){"0".into()}else{version_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")};
 let (sql,param)=if chapter.is_some(){(format!("SELECT v.id,v.version_id,bv.abbreviation,bv.language,b.name,b.book_number,v.chapter,v.verse,v.text FROM bible_verses v JOIN bible_books b ON b.id=v.book_id JOIN bible_versions bv ON bv.id=v.version_id WHERE v.version_id IN ({ids}) AND b.book_number IN (SELECT DISTINCT b2.book_number FROM bible_books b2 WHERE b2.version_id IN ({ids}) AND (b2.name LIKE ?1 OR b2.abbreviation LIKE ?1)) AND v.chapter={} {} ORDER BY b.book_number,v.chapter,v.verse,bv.language LIMIT 2000",chapter.unwrap(),verse.map(|v|format!("AND v.verse={v}")).unwrap_or_default()),format!("%{book_term}%"))}else{(format!("SELECT v.id,v.version_id,bv.abbreviation,bv.language,b.name,b.book_number,v.chapter,v.verse,v.text FROM bible_verses v JOIN bible_books b ON b.id=v.book_id JOIN bible_versions bv ON bv.id=v.version_id WHERE v.version_id IN ({ids}) AND v.text LIKE ?1 ORDER BY b.book_number,v.chapter,v.verse LIMIT 250"),format!("%{q}%"))};
 let mut s=c.prepare(&sql).map_err(err)?; let rows=s.query_map([param],|r|{let book:String=r.get(4)?;let book_number:i64=r.get(5)?;let ch:i64=r.get(6)?;let ve:i64=r.get(7)?;Ok(BibleVerse{id:r.get(0)?,version_id:r.get(1)?,version:r.get(2)?,language:r.get(3)?,book:book.clone(),book_number,chapter:ch,verse:ve,text:r.get(8)?,reference:format!("{book} {ch}:{ve}")})}).map_err(err)?; rows.map(|x|x.map_err(err)).collect()
}

#[tauri::command]
fn import_bible(path:String,db:State<Db>)->Result<i64>{ let data=fs::read_to_string(&path).map_err(err)?; let bible=parse_bible_json(&path,&data)?; let mut c=db_conn(&db); let tx=c.transaction().map_err(err)?; tx.execute("INSERT INTO bible_versions(name,abbreviation,language) VALUES(?,?,?)",params![bible.name,bible.abbreviation,bible.language]).map_err(|e|if e.to_string().contains("UNIQUE"){format!("A Bible version with abbreviation '{}' is already imported.",bible.abbreviation)}else{err(e)})?; let version_id=tx.last_insert_rowid(); for book in bible.books { tx.execute("INSERT INTO bible_books(version_id,book_number,name,abbreviation) VALUES(?,?,?,?)",params![version_id,book.number,book.name,book.abbreviation]).map_err(err)?; let book_id=tx.last_insert_rowid(); for (ci,verses) in book.chapters.into_iter().enumerate(){for(vi,text)in verses.into_iter().enumerate(){if !text.trim().is_empty(){tx.execute("INSERT INTO bible_verses(version_id,book_id,chapter,verse,text) VALUES(?,?,?,?,?)",params![version_id,book_id,ci as i64+1,vi as i64+1,text]).map_err(err)?;}}}} tx.commit().map_err(err)?; Ok(version_id) }

fn parse_bible_json(path:&str,data:&str)->Result<BibleImport>{
 let value:serde_json::Value=serde_json::from_str(data).map_err(err)?;
 if value.get("books").is_some(){return serde_json::from_value(value).map_err(err)}
 let stem=Path::new(path).file_stem().and_then(|x|x.to_str()).unwrap_or("Imported Bible").replace('_'," ");
 if let Some(items)=value.as_array(){let mut books=vec![];for(i,item)in items.iter().enumerate(){let name=item.get("name").and_then(|x|x.as_str()).unwrap_or("Book").to_string();let abbreviation=item.get("abbrev").or_else(||item.get("abbreviation")).and_then(|x|x.as_str()).map(str::to_string).unwrap_or_else(||short_name(&name));let chapters=serde_json::from_value(item.get("chapters").cloned().ok_or("A Bible book is missing chapters")?).map_err(err)?;books.push(BibleBookImport{name,abbreviation,number:i as i64+1,chapters});}let language=infer_language(path,books.first().map(|x|x.name.as_str()).unwrap_or(""));return Ok(BibleImport{name:clean_version_name(&stem),abbreviation:infer_abbreviation(&stem),language,books})}
 if let(Some(metadata),Some(verses))=(value.get("metadata"),value.get("verses").and_then(|x|x.as_array())){let mut books:Vec<BibleBookImport>=vec![];for item in verses{let number=item.get("book").and_then(|x|x.as_i64()).ok_or("Verse is missing book number")?;while books.len()<number as usize{let n=books.len() as i64+1;books.push(BibleBookImport{name:format!("Book {n}"),abbreviation:format!("B{n}"),number:n,chapters:vec![]});}let book=&mut books[number as usize-1];if let Some(name)=item.get("book_name").and_then(|x|x.as_str()){book.name=name.to_string();book.abbreviation=short_name(name);}let chapter=item.get("chapter").and_then(|x|x.as_u64()).ok_or("Verse is missing chapter")? as usize;let verse=item.get("verse").and_then(|x|x.as_u64()).ok_or("Verse is missing verse number")? as usize;while book.chapters.len()<chapter{book.chapters.push(vec![])}while book.chapters[chapter-1].len()<verse{book.chapters[chapter-1].push(String::new())}book.chapters[chapter-1][verse-1]=item.get("text").and_then(|x|x.as_str()).unwrap_or("").to_string();}let name=metadata.get("name").and_then(|x|x.as_str()).unwrap_or(&stem).to_string();let language=metadata.get("lang_short").and_then(|x|x.as_str()).map(language_name).unwrap_or_else(||infer_language(path,books.first().map(|x|x.name.as_str()).unwrap_or("")));let abbreviation=metadata.get("shortname").and_then(|x|x.as_str()).map(str::to_string).unwrap_or_else(||infer_abbreviation(&name));return Ok(BibleImport{name,abbreviation,language,books})}
 if let Some(root)=value.as_object(){let mut books=vec![];for(i,(name,chapter_value))in root.iter().enumerate(){let mut chapters=vec![];let chapter_map=chapter_value.as_object().ok_or("Book chapters must be a JSON object")?;for(_,verse_value)in chapter_map{let verse_map=verse_value.as_object().ok_or("Chapter verses must be a JSON object")?;let mut verses=vec![];for(_,text)in verse_map{verses.push(text.as_str().unwrap_or("").to_string())}chapters.push(verses)}books.push(BibleBookImport{name:name.clone(),abbreviation:short_name(name),number:i as i64+1,chapters});}let language=infer_language(path,books.first().map(|x|x.name.as_str()).unwrap_or(""));return Ok(BibleImport{name:clean_version_name(&stem),abbreviation:infer_abbreviation(&stem),language,books})}
 Err("Unsupported Bible JSON structure. Use book arrays, metadata/verses, or book/chapter/verse objects.".into())
}
fn short_name(name:&str)->String{name.chars().filter(|c|c.is_alphanumeric()).take(4).collect()}
fn clean_version_name(stem:&str)->String{stem.split_whitespace().map(|x|{let mut c=x.chars();match c.next(){Some(f)=>f.to_uppercase().collect::<String>()+c.as_str(),None=>String::new()}}).collect::<Vec<_>>().join(" ")}
fn infer_abbreviation(name:&str)->String{let upper=name.to_uppercase();if upper.contains("NEW KING JAMES")||upper.contains("NKJV")||upper.contains("NJKV"){return "NKJV".into()}if upper.contains("NEW INTERNATIONAL VERSION"){return "NIV".into()}if upper.contains("LUTHER"){return "LUTH".into()}if upper.contains("LOUIS SEGOND")||upper.contains("LSG"){return "LSG".into()}if upper.contains("SCHLACHTER"){return "SCH".into()}let compact=upper.chars().filter(|c|c.is_alphanumeric()).take(12).collect::<String>();if compact.is_empty(){"BIBLE".into()}else{compact}}
fn infer_language(path:&str,first_book:&str)->String{let p=path.to_lowercase();if p.contains("german")||p.contains("luther")||p.contains("schlachter")||p.contains("de_")||first_book.contains("Mose"){"German".into()}else if p.contains("french")||p.contains("segond")||first_book.contains("Genèse"){"French".into()}else{"English".into()}}
fn language_name(code:&str)->String{match code.to_lowercase().as_str(){"de"|"deu"|"ger"=>"German".into(),"fr"|"fra"|"fre"=>"French".into(),"en"|"eng"=>"English".into(),other=>other.to_string()}}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn parses_book_array(){let data=r#"[{"name":"Genesis","abbrev":"Gen","chapters":[["In the beginning"]]}]"#;let bible=parse_bible_json("de_schlachter.json",data).unwrap();assert_eq!(bible.books.len(),1);assert_eq!(bible.language,"German");assert_eq!(bible.books[0].chapters[0][0],"In the beginning");}
 #[test]fn parses_nested_books(){let data=r#"{"Genesis":{"1":{"1":"In the beginning"}},"Exodus":{"1":{"1":"These are the names"}}}"#;let bible=parse_bible_json("NEW_INTERNATIONAL_VERSION.json",data).unwrap();assert_eq!(bible.abbreviation,"NIV");assert_eq!(bible.books[1].number,2);}
 #[test]fn parses_metadata_verses(){let data=r#"{"metadata":{"name":"Schlachter Bibel","shortname":"SCH","lang_short":"de"},"verses":[{"book_name":"1 Mose","book":1,"chapter":1,"verse":1,"text":"Im Anfang"}]}"#;let bible=parse_bible_json("schlachter.json",data).unwrap();assert_eq!(bible.language,"German");assert_eq!(bible.books[0].chapters[0][0],"Im Anfang");}
}
#[tauri::command]
fn delete_bible_version(id:i64,db:State<Db>)->Result<()>{db_conn(&db).execute("DELETE FROM bible_versions WHERE id=?",[id]).map_err(err)?;Ok(())}

#[tauri::command]
fn list_media(db:State<Db>)->Result<Vec<MediaFile>>{let c=db_conn(&db);let mut s=c.prepare("SELECT id,name,file_name,path,media_type,is_default FROM media_files ORDER BY is_default DESC,name").map_err(err)?;let out=s.query_map([],|r|Ok(MediaFile{id:r.get(0)?,name:r.get(1)?,file_name:r.get(2)?,path:r.get(3)?,media_type:r.get(4)?,is_default:r.get::<_,i64>(5)?==1})).map_err(err)?.map(|x|x.map_err(err)).collect();out}
#[tauri::command]
fn import_media(source:String,app:AppHandle,db:State<Db>)->Result<i64>{let src=Path::new(&source);let ext=src.extension().and_then(|x|x.to_str()).unwrap_or("").to_lowercase();let media_type=if ["mp4","webm"].contains(&ext.as_str()){"video"}else if ["jpg","jpeg","png"].contains(&ext.as_str()){"image"}else{return Err("Supported media: JPG, PNG, MP4, WebM".into())};let dir=app.path().app_data_dir().map_err(err)?.join("media");fs::create_dir_all(&dir).map_err(err)?;let original=src.file_name().and_then(|x|x.to_str()).ok_or("Invalid file name")?;let stored=format!("{}-{}",chrono::Utc::now().timestamp_millis(),original);let dest=dir.join(&stored);fs::copy(src,&dest).map_err(err)?;let name=src.file_stem().and_then(|x|x.to_str()).unwrap_or("Background");let c=db_conn(&db);c.execute("INSERT INTO media_files(name,file_name,path,media_type) VALUES(?,?,?,?)",params![name,stored,dest.to_string_lossy(),media_type]).map_err(err)?;Ok(c.last_insert_rowid())}
#[tauri::command]
fn delete_media(id:i64,db:State<Db>)->Result<()>{let c=db_conn(&db);let path:Option<String>=c.query_row("SELECT path FROM media_files WHERE id=?",[id],|r|r.get(0)).optional().map_err(err)?;c.execute("DELETE FROM media_files WHERE id=?",[id]).map_err(err)?;if let Some(p)=path{let _=fs::remove_file(p);}Ok(())}
#[tauri::command]
fn set_default_media(id:Option<i64>,db:State<Db>)->Result<()>{let mut c=db_conn(&db);let tx=c.transaction().map_err(err)?;tx.execute("UPDATE media_files SET is_default=0",[]).map_err(err)?;if let Some(id)=id{tx.execute("UPDATE media_files SET is_default=1 WHERE id=?",[id]).map_err(err)?;}tx.execute("UPDATE projection_settings SET default_media_id=? WHERE id=1",[id]).map_err(err)?;tx.commit().map_err(err)?;Ok(())}

#[tauri::command]
fn get_projection_settings(db:State<Db>)->Result<ProjectionSettings>{db_conn(&db).query_row("SELECT font_family,font_size,text_color,alignment,overlay_opacity,transition_ms,default_media_id FROM projection_settings WHERE id=1",[],|r|Ok(ProjectionSettings{font_family:r.get(0)?,font_size:r.get(1)?,text_color:r.get(2)?,alignment:r.get(3)?,overlay_opacity:r.get(4)?,transition_ms:r.get(5)?,default_media_id:r.get(6)?})).map_err(err)}
#[tauri::command]
fn save_projection_settings(s:ProjectionSettings,db:State<Db>)->Result<()>{db_conn(&db).execute("UPDATE projection_settings SET font_family=?,font_size=?,text_color=?,alignment=?,overlay_opacity=?,transition_ms=?,default_media_id=? WHERE id=1",params![s.font_family,s.font_size,s.text_color,s.alignment,s.overlay_opacity,s.transition_ms,s.default_media_id]).map_err(err)?;Ok(())}
#[tauri::command]
fn export_database(path:String,app:AppHandle)->Result<()>{let src=app.path().app_data_dir().map_err(err)?.join("churchprojection.db");fs::copy(src,path).map_err(err)?;Ok(())}
#[tauri::command]
fn open_projection(app:AppHandle)->Result<()>{create_projection_window(&app)}
fn create_projection_window(app:&AppHandle)->Result<()>{if let Some(w)=app.get_webview_window("projection"){w.show().map_err(err)?;w.set_focus().map_err(err)?;return Ok(())}WebviewWindowBuilder::new(app,"projection",WebviewUrl::App("projection.html".into())).title("Projection Display").inner_size(1280.0,720.0).decorations(true).visible(true).focused(true).build().map_err(err)?;Ok(())}
#[tauri::command]
fn set_projection_fullscreen(app:AppHandle,fullscreen:bool)->Result<()>{let w=app.get_webview_window("projection").ok_or("Projection window is not open")?;w.set_fullscreen(fullscreen).map_err(err)}
#[tauri::command]
fn publish_projection(payload:serde_json::Value,state:State<ProjectionStore>)->Result<()>{*state.0.lock().map_err(err)?=Some(payload);Ok(())}
#[tauri::command]
fn get_projection_state(state:State<ProjectionStore>)->Result<Option<serde_json::Value>>{Ok(state.0.lock().map_err(err)?.clone())}
#[tauri::command]
fn get_app_version()->String{env!("CARGO_PKG_VERSION").to_string()}
#[tauri::command]
fn install_update(path:String,app:AppHandle)->Result<()>{let file=Path::new(&path);let name=file.file_name().and_then(|x|x.to_str()).unwrap_or("");if !file.is_file()||file.extension().and_then(|x|x.to_str()).map(|x|!x.eq_ignore_ascii_case("exe")).unwrap_or(true)||!name.starts_with("ChurchProjection Studio_"){return Err("Select a ChurchProjection Studio NSIS update installer.".into())}Command::new(file).arg("/S").spawn().map_err(err)?;std::thread::spawn(move||{std::thread::sleep(std::time::Duration::from_millis(700));app.exit(0);});Ok(())}
#[tauri::command]
fn report_frontend_error(message:String,app:AppHandle)->Result<()>{let path=app.path().app_data_dir().map_err(err)?.join("frontend-error.log");fs::write(path,message).map_err(err)}
#[tauri::command]
fn report_projection_status(title:String,app:AppHandle)->Result<()>{if std::env::var_os("CPS_PROJECTION_TEST").is_some(){let path=app.path().app_data_dir().map_err(err)?.join("projection-test.log");fs::write(path,format!("READY: {title}")).map_err(err)?;}Ok(())}
fn err<E:std::fmt::Display>(e:E)->String{e.to_string()}

fn seed(c:&Connection)->rusqlite::Result<()> { let count:i64=c.query_row("SELECT count(*) FROM songs",[],|r|r.get(0))?;if count==0{c.execute("INSERT INTO songs(title,author,category,language,copyright_info) VALUES('Amazing Grace','John Newton','Hymn','English','Public domain')",[])?;let id=c.last_insert_rowid();for(pos,(kind,label,text))in[("verse","Verse 1","Amazing grace, how sweet the sound\nThat saved a soul like me\nI once was lost, but now am found\nWas blind, but now I see"),("verse","Verse 2","'Twas grace that taught my heart to fear\nAnd grace my fears relieved\nHow precious did that grace appear\nThe hour I first believed")].iter().enumerate(){c.execute("INSERT INTO song_sections(song_id,section_type,label,content,position) VALUES(?,?,?,?,?)",params![id,kind,label,text,pos])?;}}Ok(())}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){let bootstrap=Connection::open_in_memory().expect("failed to create bootstrap database");bootstrap.execute_batch(include_str!("schema.sql")).expect("failed to initialize bootstrap database");seed(&bootstrap).expect("failed to seed bootstrap database");tauri::Builder::default().manage(Db(Mutex::new(bootstrap))).manage(ProjectionStore(Mutex::new(None))).plugin(tauri_plugin_dialog::init()).setup(|app|{let dir=app.path().app_data_dir()?;fs::create_dir_all(&dir)?;let conn=Connection::open(dir.join("churchprojection.db"))?;conn.execute_batch(include_str!("schema.sql"))?;seed(&conn)?;*app.state::<Db>().0.lock().expect("database mutex poisoned")=conn;let _=app.emit_to("main","database:ready",());if std::env::var_os("CPS_PROJECTION_TEST").is_some(){create_projection_window(app.handle()).map_err(std::io::Error::other)?;}Ok(())}).invoke_handler(tauri::generate_handler![dashboard_stats,list_songs,save_song,delete_song,import_song_text,export_songs,list_bible_versions,list_bible_books,bible_chapter_count,bible_verse_count,search_bible,import_bible,delete_bible_version,list_media,import_media,delete_media,set_default_media,get_projection_settings,save_projection_settings,export_database,open_projection,set_projection_fullscreen,publish_projection,get_projection_state,get_app_version,install_update,report_frontend_error,report_projection_status]).run(tauri::generate_context!()).expect("error while running ChurchProjection Studio");}
