use super::LogSignal;
use super::UsageBlock;
use hdk::prelude::holo_hash::*;
use hdk::prelude::*;

enum UsageBlockLinkType {
    BlockToTime = 0,
    AgentToPath,
}

impl From<UsageBlockLinkType> for LinkType {
    fn from(hdk_link_type: UsageBlockLinkType) -> Self {
        Self(hdk_link_type as u8)
    }
}

#[hdk_extern]
pub fn subscribe(_: ()) -> ExternResult<()> {
    let agent_info = agent_info()?;
    let agent_address: AnyDhtHash = agent_info.agent_initial_pubkey.clone().into();
    let path = Path::from("Agents");
    path.ensure()?;
    create_link(
        path.path_entry_hash()?,
        agent_address.into(),
        UsageBlockLinkType::AgentToPath,
        link_tag("")?,
    )?;
    Ok(())
}

#[hdk_extern]
pub fn get_usage_block(entry_hash: EntryHashB64) -> ExternResult<Option<UsageBlock>> {
    let maybe_element = get(EntryHash::from(entry_hash), GetOptions::default())?;

    match maybe_element {
        None => Ok(None),
        Some(element) => {
            let usage_block: UsageBlock = element.entry().to_app_option()?.ok_or(
                WasmError::Guest("Could not deserialize element to UsageBlock.".into()),
            )?;

            Ok(Some(usage_block))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewUsageBlockOutput {
    header_hash: HeaderHashB64,
    entry_hash: EntryHashB64,
}

#[hdk_extern]
pub fn create_usage_block(usage_block: UsageBlock) -> ExternResult<NewUsageBlockOutput> {
    let header_hash = create_entry(&usage_block)?;

    let entry_hash = hash_entry(&usage_block)?;

    let path = Path::from(&usage_block.t);
    path.ensure()?;

    create_link(
        path.path_entry_hash()?,
        entry_hash.clone(),
        UsageBlockLinkType::BlockToTime,
        link_tag("")?,
    )?;

    let output = NewUsageBlockOutput {
        header_hash: HeaderHashB64::from(header_hash),
        entry_hash: EntryHashB64::from(entry_hash),
    };
    let agents_path = Path::from("Agents");
    agents_path.ensure()?;
    let links = get_links(agents_path.path_entry_hash()?, None)?;
    let agents = links.into_iter().map(|link| link.target.into()).collect();
    debug!("{:?}", agents);
    let log_signal = LogSignal {
        t: usage_block.t,
        l: usage_block.l,
        s: usage_block.s,
        b: usage_block.b,
        i: usage_block.i,
        g: usage_block.g,
    };
    let signal = ExternIO::encode(log_signal)?;
    remote_signal(signal, agents)?;
    Ok(output)
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
struct StringLinkTag(String);
pub fn link_tag(tag: &str) -> ExternResult<LinkTag> {
    let sb: SerializedBytes = StringLinkTag(tag.into()).try_into()?;
    Ok(LinkTag(sb.bytes().clone()))
}
