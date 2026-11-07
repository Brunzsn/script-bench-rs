let value = 1.0;
for _i in 0..5000000 {
    value = value * 0.999999 + 0.000001;
}
value
