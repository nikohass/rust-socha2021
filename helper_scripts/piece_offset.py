
shape = int(
    "000000000000000000000" +
    "000000000000000000011" +
    "000000000000000000010" +
    "000000000000000000010" +
    "000000000000000000010",
    base=2
)

points = "000000000000000000000" + \
         "000000000000000000011" + \
         "000000000000000000000" + \
         "000000000000000000000" + \
         "000000000000000000010"

offsets = []
points = "0" * (128 - len(points)) + points
for idx, char in enumerate(points):
    if char == "1":
        offsets.append(127 - idx)

print("offsets:", sorted(offsets))
print("shape:", shape)
