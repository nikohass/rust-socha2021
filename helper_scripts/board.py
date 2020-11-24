import pygame
pygame.display.init()
window = pygame.display.set_mode((420, 420))
pygame.display.set_caption("Board")

def draw(ones):
    window.fill((255, 255, 255))
    index = 0
    for j in range(21):
        for i in range(21):
            color = ((0, 255, 0) if j < 20 and i < 20 else (255, 0, 0)) if index in ones else (0, 0, 0)
            pygame.draw.rect(window, color, (i * 20 + 1, j * 20 + 1, 18, 18))
            index += 1
    pygame.display.update()

def to_bitboard(ones):
    i = 0
    for idx in ones:
        i |= 1 << idx
    print("one:", i >> 384 & 340282366920938463463374607431768211455)
    print("two:", i >> 256 & 340282366920938463463374607431768211455)
    print("three:", i >> 128 & 340282366920938463463374607431768211455)
    print("four:", i & 340282366920938463463374607431768211455)

def main():
    ones = []
    while True:
        pygame.time.delay(60)
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                return
        if pygame.mouse.get_pressed()[0]:
            x, y = pygame.mouse.get_pos()
            idx = int(x / 20) + int(y / 20) * 21
            if not idx in ones:
                ones.append(idx)
            to_bitboard(ones)

        if pygame.mouse.get_pressed()[2]:
            x, y = pygame.mouse.get_pos()
            idx = int(x / 20) + int(y / 20) * 21
            if idx in ones:
                ones.remove(idx)
            to_bitboard(ones)
        draw(ones)

if __name__ == "__main__":
    main()
