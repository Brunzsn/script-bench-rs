// allow(L001)
use bench.data

on run() -> int {
    let count = 5000
    let unicode_text = "héllo—世界🚀"

    var parts: list[str] = []
    var index = 0
    while index < count {
        parts.push("item-{index}")
        index += 1
    }

    let joined = parts.join(",")
    let split_parts = joined.split(",")
    let pipe_joined = split_parts.join("|")
    let replaced = pipe_joined.replace("item-", "row-")
    let marked = replaced.replace("row-4999", "last")
    let unicode_length = unicode_text.chars().len()

    var checksum =
        joined.len() + pipe_joined.len() + marked.len() + split_parts.len() + unicode_length
    if marked.contains("last") { checksum += 1 }
    checksum
}
