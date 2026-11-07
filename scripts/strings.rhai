let count = 5000;
let unicode_text = "héllo—世界🚀";

let parts = [];
for index in 0..count {
    parts.push("item-" + index);
}

let joined = "";
for index in 0..parts.len {
    if index > 0 {
        joined += ",";
    }
    joined += parts[index];
}

let split_parts = joined.split(",");
let pipe_joined = "";
for index in 0..split_parts.len {
    if index > 0 {
        pipe_joined += "|";
    }
    pipe_joined += split_parts[index];
}

let replaced = pipe_joined;
replaced.replace("item-", "row-");
let marked = replaced;
marked.replace("row-4999", "last");
let unicode_length = unicode_text.len;

let checksum = joined.len + pipe_joined.len + marked.len + split_parts.len + unicode_length;
if marked.contains("last") {
    checksum += 1;
}
checksum
