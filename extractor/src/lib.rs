use anyhow::{anyhow, Result};
use parse_wiki_text::{Configuration, Node};
use serde_json::Value;

pub fn fetch_wiki_page(url: &str, page: &str) -> Result<Value> {
    let url = reqwest::Url::parse_with_params(url, &[
        ("action", "parse"),
        ("page", page),
        ("format", "json"),
        ("prop", "wikitext"),
    ])?;
    let response = reqwest::blocking::get(url)?;
    Ok(response.json::<Value>()?)
}

pub fn extract_wikitext(value: &Value) -> Option<&str> {
    value.get("parse")
        .and_then(|p| p.get("wikitext"))
        .and_then(|w| w.get("*"))
        .and_then(|a| a.as_str())
}

pub fn create_configuration() -> ::parse_wiki_text::Configuration {
    Configuration::new(&parse_wiki_text::ConfigurationSource {
        category_namespaces: &[
            "category",
        ],
        extension_tags: &[
            "gallery",
            "indicator",
            "nowiki",
            "pre",
            "ref",
            "references",
            "source",
            "syntaxhighlight",
        ],
        file_namespaces: &[
            "file",
            "image",
        ],
        link_trail: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        magic_words: &[
            "EXPECTUNUSEDCATEGORY",
            "FORCETOC",
            "HIDDENCAT",
            "INDEX",
            "NEWSECTIONLINK",
            "NOCC",
            "NOCONTENTCONVERT",
            "NOEDITSECTION",
            "NOGALLERY",
            "NOINDEX",
            "NONEWSECTIONLINK",
            "NOTC",
            "NOTITLECONVERT",
            "NOTOC",
            "STATICREDIRECT",
            "TOC",
        ],
        protocols: &[
            "//",
            "bitcoin:",
            "ftp://",
            "ftps://",
            "geo:",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc://",
            "ircs://",
            "magnet:",
            "mailto:",
            "mms://",
            "news:",
            "nntp://",
            "redis://",
            "sftp://",
            "sip:",
            "sips:",
            "sms:",
            "ssh://",
            "svn://",
            "tel:",
            "telnet://",
            "urn:",
            "worldwind://",
            "xmpp:",
        ],
        redirect_magic_words: &[
            "REDIRECT",
        ]
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use super::*;
    #[test]
    fn test_fetch_tables() -> Result<()> {
        let response = fetch_wiki_page("https://wiki.vg/api.php", "Protocol")?;
        let wiki_text = extract_wikitext(&response)
            .ok_or(anyhow!("WikiText not present"))?;

        let result = create_configuration().parse(wiki_text);

        result.nodes
            .iter()
            .filter_map(|n| match n {
                Node::Table {
                    rows,
                    ..
                } => {
                    rows.get(0)
                        .and_then(|r| r.cells.get(0))
                        .and_then(|c| c.content.get(0))
                        .filter(|n| match n {
                            Node::Text {
                                value: "Packet ID",
                                ..
                            } => true,
                            _ => false,
                        })
                        .and(Some(rows))
                },
                _ => None,
            })
            .for_each(|n| println!("{:#?}", n));

        Ok(())
    }
}
