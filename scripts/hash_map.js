const count = 20000;
const modulus = 1000000007;

function bench() {
    const map = new Map();
    for (let index = 0; index < count; index++) {
        map.set(`key${index}`, index * 3);
    }

    let lookedUp = 0;
    for (let index = 0; index < count; index++) {
        lookedUp += map.get(`key${index}`);
    }

    let iterated = 0;
    for (const value of map.values()) {
        iterated += value;
    }

    return (lookedUp + iterated + map.size) % modulus;
}
