use super::offers::{self, GeneralFrameType, COMMON_FILENAME, NETEX_COMMON};
use failure::format_err;
use minidom::Element;
use minidom_ext::{AttributeElementExt, OnlyChildElementExt};
use transit_model::{
    netex_utils::{self, FrameType},
    objects::{Comment, CommentType},
    Result,
};
use typed_index_collection::CollectionWithId;

pub fn parse_common(common: &Element) -> Result<CollectionWithId<Comment>> {
    let frames = netex_utils::parse_frames_by_type(common.try_only_child("dataObjects")?)?;
    let general_frames = frames
        .get(&FrameType::General)
        .ok_or_else(|| format_err!("Failed to find a GeneralFrame in {}", COMMON_FILENAME))?;
    let general_frames_by_type = offers::parse_general_frame_by_type(general_frames)?;
    let common_general_frame = general_frames_by_type
        .get(&GeneralFrameType::Common)
        .ok_or_else(|| format_err!("Failed to find the GeneralFrame of type {}", NETEX_COMMON))?;
    let comments = common_general_frame
        .only_child("members")
        .iter()
        .flat_map(|members| members.children())
        .filter_map(|notice_element| {
            let id = notice_element.attribute::<String>("id")?;
            let name = notice_element.only_child("Text")?.text().trim().to_string();
            let comment = Comment {
                id,
                name,
                comment_type: CommentType::Information,
                label: None,
                url: None,
            };
            Some(comment)
        })
        .collect();
    Ok(CollectionWithId::new(comments)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_comments() {
        let xml = r#"<PublicationDelivery>
                <dataObjects>
                    <GeneralFrame>
                        <TypeOfFrameRef ref="NETEX_COMMUN" />
                        <members>
                            <Notice id="comment_id">
                                <Text>Comment</Text>
                            </Notice>
                            <!-- This Notice is discarded because it doesn't have an `id` attribute -->
                            <Notice>
                                <Text>Comment</Text>
                            </Notice>
                            <!-- This Notice is discarded because it doesn't have a `Text` child -->
                            <Notice id="comment_id" />
                        </members>
                    </GeneralFrame>
                </dataObjects>
            </PublicationDelivery>"#;
        let root: Element = xml.parse().unwrap();
        let comments = parse_common(&root).unwrap();
        assert_eq!(1, comments.len());
        let comment = comments.get("comment_id").unwrap();
        assert_eq!("Comment", comment.name);
    }
}
