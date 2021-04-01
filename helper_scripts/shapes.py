SHAPES = [1, 3, 2097153, 7, 4398048608257, 15, 9223376434903384065, 31, 19342822337210501698682881, 6291459, 8796107702274, 6291457, 2097155, 4194307, 6291458, 4398048608259, 8796097216515, 13194143727618, 13194141630465, 14680065, 2097159, 8388615, 14680068, 16777231, 2097167, 31457281, 31457288, 9223376434903384067, 27670120508612935681, 18446752869806768131, 27670124906661543938, 8796097216519, 30786329772034, 4398061191169, 17592200724484, 4194311, 14680066, 4398052802561, 8796099313666, 6291462, 12582915, 4398052802562, 8796099313665, 17592200724481, 4398061191172, 13194143727622, 26388283260931, 10485767, 14680069, 13194141630467, 13194143727619, 13194152116226, 26388285358082, 8796099313670, 8796105605123, 8796107702276, 8796107702273, 17592200724482, 4398061191170, 26388285358081, 13194152116228, 17592198627331, 4398052802566, 9223385230998503426, 18446757267851182081, 9223376434907578370, 18446752869808865281, 14680076, 25165831, 29360131, 6291470, 4398048608263, 30786333966340, 17592194433031, 30786327674881, 4398052802563, 8796099313667, 14680067, 6291463, 12582919, 14680070, 13194145824770, 13194145824769, 9223376434907578369, 9223385230996406273, 18446757267853279234, 18446752869808865282, 8388623, 4194319, 31457284, 31457282]

def get_l(shape):
    bit_index = 0
    l = []
    while shape != 0:
        bit = 1 << bit_index
        if shape & bit != 0:
            shape ^= bit
            l.append(bit_index)
        bit_index += 1
    return l

def get_p(shape):
    shape <<= 128
    shape_copy = shape
    p = []
    bit_index = 1
    while shape_copy != 0:
        bit = 1 << bit_index
        if bit & shape_copy != 0:
            shape_copy ^= bit
            diagonal_neighbours = bit << 20 | bit << 22 | bit >> 20 | bit >> 22
            bit_idx = 1
            while diagonal_neighbours != 0:
                bit = 1 << bit_idx
                if bit & diagonal_neighbours != 0:
                    diagonal_neighbours ^= bit
                    neighbours = bit << 21 | bit >> 21 | bit << 1 | bit >> 1
                    if neighbours & shape == 0:
                        p.append(bit_index - 128)
                        break
                bit_idx += 1
        bit_index += 1
    return p

for shape_index, shape in enumerate(SHAPES):
    string = f"fn shape_{shape_index}(l: Bitboard, p: Bitboard) -> Bitboard"
    string += " {\n    "
    l = get_l(shape)
    p = get_p(shape)
    if len(p) == 1:
        l = []
    for i, shift in enumerate(l):
        if i == 0:
            if len(l) != 1:
                string += "("
        else:
            string += " & "
        if shift == 0:
            string += "l"
        else:
            string += f"l >> {shift}"
    if len(l) != 1 and len(p) > 1:
        string += ")"

    for i, shift in enumerate(p):
        if i == 0:
            if len(p) != 1:
                string += " & "
                string += "("
        else:
            string += " | "
        if shift == 0:
            string += "p"
        else:
            string += f"p >> {shift}"
    if len(p) != 1:
        string += ")"
    string += "\n}\n"
    print(string)
