use std::fs::File;
use std::io::{self,BufRead};
use std::collections::HashMap;
use quick_xml::events::attributes::Attributes;
use quick_xml::name::QName;
use quick_xml::reader::{Reader};
use quick_xml::events::{Event};
use std::borrow::Cow;

use egui_rss::channel::{Channel, ChannelBuilder, RSSVersion};
use egui_rss::item::Item;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    
    let urls = read_urls_from_file(&"urls.txt".to_string()).unwrap();

    for url in urls.iter() {
        println!("{url:#?}");
        match get_text(&url).await {
            Ok(content) => {
                match parse_xml_to_channel(&content) {
                    Ok(channel) => {
                        println!("{:?}",channel);
                        //todo we get a channel now
                        //feeds.insert(&channel.title, channel);
                    },
                    Err(error) => println!("xml parse error for \'{url}\': {error}"),
                }
                
            },
            Err(error) => println!("read error for \'{url}\': {error}")
        }

    }
    Ok(())
}

fn parse_xml_to_channel(rss_string: &String) -> Result<Channel, &'static str>{
    let mut parser = Reader::from_str(&rss_string);
    parser.trim_text(true);

    let mut channel_builder = ChannelBuilder::new();

    //match rss tag, enusre v2.0  
    let attributes:HashMap<String, String> = scan_to("rss", &mut parser)?;
    match attributes.get(&String::from("version")){
        Some(version) => {
            //TODO parse this out, no hardcode
            println!("{version}");
            if version=="2.0" {
                channel_builder.set_version(&RSSVersion{ major: 2, minor: 0 });
            }
        },
        None => return Err("no version found"),
    };


    _ = scan_to("channel", &mut parser)?;
    let mut inside_tag: bool = false;
    let mut opened_tag: String = String::from("");
    let mut opened_tag_depth: i32 = -1;
    let mut inner_xml: String = String::from("");
    let mut depth: i32 = 0;

    loop{
        match parser.read_event(){
            Ok(Event::End(e)) if e.name().as_ref() == b"channel" =>{
                let channel:Channel = channel_builder.build()?;
                return Ok(channel)
            },
            Ok(Event::End(e)) => {
                println!("3{opened_tag}");
                if depth == opened_tag_depth && opened_tag == String::from_utf8_lossy(e.name().as_ref()){
                    
                    if opened_tag == "link"{
                        channel_builder.set_link(&inner_xml);
                    }
                    else if opened_tag == "description"{
                        channel_builder.set_description(&inner_xml);
                    }
                    else if opened_tag == "title"{
                        channel_builder.set_title(&inner_xml);
                    }
                    inside_tag = false;
                    opened_tag.clear();
                    inner_xml.clear();
                    depth -= 1;
                }
                else{
                    return Err("not inside tag, or not a matching close tag. todo split up these errors.");
                }
            }

            Ok(Event::Start(ref e)) => {
                println!("1{opened_tag}");

                opened_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                inside_tag = true;
                depth += 1;
                opened_tag_depth = depth;
                println!("2{opened_tag}");

                //skip item
                if opened_tag == "item" {
                    let _ = parser.read_to_end(QName(b"item"));

                }

            }

            Ok(Event::Text(e)) => {
                inner_xml = e.unescape().unwrap().into_owned()
            },
            Err(_error) => return Err("some error"),
            Ok(_) => ()
        }
    }
    
    //todo fix channel and item defs

    //for each <item> build an Item
    
}

fn qname_to_string(qname: &QName) -> String{
    String::from_utf8_lossy(qname.0).to_string()
}

///Consume XML tags until the specified tag OPEN is found. Returns the tag attributes as a hashmap.
fn scan_to<'a>(tag: &str, parser:&'a mut quick_xml::Reader<&[u8]>) -> Result<HashMap<String, String>, &'static str>{
    loop {
        match parser.read_event(){
            Ok(Event::Start(e)) => {

                let tag_name = QName(tag.as_bytes());
                if e.name().eq(&tag_name) {
                    let mut a_map = HashMap::new();
                    for a in e.attributes() {
                        match a {
                            Ok(kvp) => {
                                let k:String = qname_to_string(&kvp.key);
                                let v:String = String::from_utf8_lossy(kvp.value.as_ref()).to_string();
                                a_map.insert(k,v);
                            }
                            Err(_) => {
                                return Err("Some error lol");
                            }
                        }
                    }
                    return Ok(a_map);
                }
            },
            Ok(Event::Eof) => {
                break;
            },
            Err(_) => {
                return Err("Some error lol");
            },
            _ => (),

        }
    }
    return Err("Some xml parse error lol");
}

async fn get_text(url: &String) -> reqwest::Result<String>{
    match reqwest::get::<&String>(&url).await{
        Ok(resp) => resp.text().await, //future<Result>
        Err(error) => Err(error),      //Result::Err
    }
}

fn read_urls_from_file(filepath: &String) -> std::io::Result<Vec<String>>{
    let file = File::open(filepath)?;
    let reader = io::BufReader::new(file);

    let urls: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();
    Ok(urls)
}