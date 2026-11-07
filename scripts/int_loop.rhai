let modulus = 1000000007;
let value = 12345;
for index in 0..10000000 {
    value = (value * 31 + index) % modulus;
}
value
