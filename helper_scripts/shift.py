import pygame
pygame.display.init()
window = pygame.display.set_mode((420, 420))

def draw(start, end):
    window.fill((255, 255, 255))
    if start > end - 2:
        c = (255, 0, 0)
        start, end = (end - 2, start + 2)
    else:
        c = (0, 255, 0)

    index = 0
    for j in range(21):
        for i in range(21):
            color = c if index > start and index < end else (0, 0, 0)
            pygame.draw.rect(window, color, (i * 20 + 1, j * 20 + 1, 18, 18))
            index += 1
    pygame.display.update()

def main():
    first = True
    start = end = -2
    while True:
        pygame.time.delay(50)
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                return
        if pygame.mouse.get_pressed()[0]:
            x, y = pygame.mouse.get_pos()
            if first:
                start = x // 20 + y // 20 * 21 - 1
                end = x // 20 + y // 20 * 21 + 1
            else:
                end = x // 20 + y // 20 * 21 + 1
                shift = end - start - 2
                pygame.display.set_caption((">> " if shift > 0 else "<< ") + str(abs(shift)))
            first = not first
            pygame.time.delay(150)
        draw(start, end)

if __name__ == "__main__":
    main()
