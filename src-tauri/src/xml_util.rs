use anyhow::Result;
use quick_xml::{
    events::{attributes::Attribute, BytesStart, Event},
    name::QName,
    Reader, Writer,
};
use std::{borrow::Cow, io::Cursor};

pub fn xml_find_value(
    xml_content: &str,
    node_tree: &[&str],
    attr_key: &str,
) -> Result<Vec<String>> {
    let mut name_list: Vec<String> = vec![];

    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut node_tree_index = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // exchange package name
                if e.name().as_ref() == node_tree[node_tree_index].as_bytes() {
                    if node_tree_index == node_tree.len() - 1 {
                        let value = String::from_utf8(
                            e.attributes()
                                .collect::<Result<Vec<Attribute<'_>>, _>>()?
                                .iter()
                                .find(|attr| attr.key.as_ref() == attr_key.as_bytes())
                                .unwrap_or(&Attribute {
                                    key: QName(b""),
                                    value: Cow::Borrowed(b""),
                                })
                                .value
                                .to_vec(),
                        )?;
                        name_list.push(value);
                    } else {
                        node_tree_index += 1;
                    }
                }
            }
            Ok(Event::End(_e)) => {
                node_tree_index = node_tree_index.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(_e) => {}
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
        }
    }

    Ok(name_list)
}

pub fn xml_exchange_value(
    xml_content: &str,
    node_tree: &[&str],
    attr_key: &str,
    attr_value: &str,
) -> Result<String> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut node_tree_index = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                // exchange package name
                if e.name().as_ref() == node_tree[node_tree_index].as_bytes() {
                    if node_tree_index == node_tree.len() - 1 {
                        let mut elem = BytesStart::new(String::from_utf8_lossy(e.name().0));
                        elem.extend_attributes(
                            e.attributes()
                                .collect::<Result<Vec<Attribute<'_>>, _>>()?
                                .iter()
                                .filter(|attr| attr.key.as_ref() != attr_key.as_bytes())
                                .map(|attr| attr.to_owned()),
                        );
                        elem.push_attribute((attr_key, attr_value));
                        assert!(writer.write_event(Event::Start(elem)).is_ok());
                    } else {
                        node_tree_index += 1;
                    }
                } else {
                    assert!(writer.write_event(Event::Start(e)).is_ok());
                }
            }
            Ok(Event::End(e)) => {
                assert!(writer.write_event(Event::End(e)).is_ok());
                node_tree_index = node_tree_index.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ))
            }
        }
    }

    Ok(String::from_utf8(writer.into_inner().into_inner()).unwrap())
}
