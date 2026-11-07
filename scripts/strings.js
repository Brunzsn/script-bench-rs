const count = 5000;
const unicodeText = "héllo—世界🚀";

function bench() {
    const parts = [];
    for (let index = 0; index < count; index++) {
        parts.push(`item-${index}`);
    }

    const joined = parts.join(",");
    const splitParts = joined.split(",");
    const pipeJoined = splitParts.join("|");
    const replaced = pipeJoined.split("item-").join("row-");
    const marked = replaced.replace("row-4999", "last");
    let unicodeLength = 0;
    for (const character of unicodeText) {
        unicodeLength += 1;
    }

    let checksum = joined.length + pipeJoined.length + marked.length + splitParts.length + unicodeLength;
    if (marked.indexOf("last") >= 0) {
        checksum += 1;
    }
    return checksum;
}
