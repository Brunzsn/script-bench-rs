let count = 20000;
let modulus = 1000000007;

let map = #{};
for index in 0..count {
    map["key" + index] = index * 3;
}

let looked_up = 0;
for index in 0..count {
    looked_up += map["key" + index];
}

let iterated = 0;
for value in map.values() {
    iterated += value;
}

(looked_up + iterated + map.len()) % modulus
