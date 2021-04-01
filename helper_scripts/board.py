import pygame
pygame.display.init()
window = pygame.display.set_mode((630, 630))
pygame.display.set_caption("Board")
pygame.font.init()
font = pygame.font.SysFont("arial", 13)

RED = (255, 0, 0)
GREEN = (0, 255, 0)
BLACK = (0, 0, 0)

def draw(board):
    window.fill((255, 255, 255))
    for y in range(21):
        for x in range(21):
            bit = 1 << (x + y * 21)
            color = BLACK
            if bit & board != 0:
                if y > 19 or x > 19:
                    color = RED
                else:
                    color = GREEN
            pygame.draw.rect(window, color, (x * 30 + 1, y * 30 + 1, 28, 28))
            window.blit(font.render(str(x + y * 21), 1, (255, 255, 255)), (x * 30 + 2, y * 30 + 5))
    pygame.display.update()

def field_at_coords(x, y):
    return x // 30 + y // 30 * 21

def main():
    board = 0
    ones = []
    while True:
        pygame.time.delay(10)
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                return
        if pygame.mouse.get_pressed()[0]:
            x, y = pygame.mouse.get_pos()
            shift = field_at_coords(x, y)
            if shift >= 0 and shift <= 440:
                board |= 1 << shift
                print(board)

        if pygame.mouse.get_pressed()[2]:
            x, y = pygame.mouse.get_pos()
            shift = field_at_coords(x, y)
            if shift >= 0 and shift <= 440:
                board &= ~(1 << shift)
                print(board)
        draw(board)

if __name__ == "__main__":
    main()
