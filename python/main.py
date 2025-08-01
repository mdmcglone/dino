import pygame
import math
from typing import Tuple
from maps.pangaea import PangaeaMap
from maps.terrain import TerrainType

# Initialize Pygame
pygame.init()

# Screen dimensions
SCREEN_WIDTH = 1400
SCREEN_HEIGHT = 900
HEX_SIZE = 25  # Radius of hexagon

class HexMapRenderer:
    """Handles rendering of hex maps with neon aesthetic"""
    
    def __init__(self, hex_map):
        self.hex_map = hex_map
        self.camera_x = 0
        self.camera_y = 0
    
    def hex_to_pixel(self, q: int, r: int) -> Tuple[float, float]:
        """Convert hex coordinates to pixel coordinates"""
        x = HEX_SIZE * 3/2 * q
        y = HEX_SIZE * math.sqrt(3) * (r + q/2)
        return x + 50 - self.camera_x, y + 50 - self.camera_y
    
    def pixel_to_hex(self, x: float, y: float) -> Tuple[int, int]:
        """Convert pixel coordinates to hex coordinates"""
        # Adjust for camera and offset
        x = x - 50 + self.camera_x
        y = y - 50 + self.camera_y
        
        # Convert to hex coordinates
        q = (2/3 * x) / HEX_SIZE
        r = (-1/3 * x + math.sqrt(3)/3 * y) / HEX_SIZE
        
        # Round to nearest hex
        return self._hex_round(q, r)
    
    def _hex_round(self, q: float, r: float) -> Tuple[int, int]:
        """Round fractional hex coordinates to nearest hex"""
        s = -q - r
        rq = round(q)
        rr = round(r)
        rs = round(s)
        
        q_diff = abs(rq - q)
        r_diff = abs(rr - r)
        s_diff = abs(rs - s)
        
        if q_diff > r_diff and q_diff > s_diff:
            rq = -rr - rs
        elif r_diff > s_diff:
            rr = -rq - rs
            
        return int(rq), int(rr)
    
    def draw_hex(self, screen: pygame.Surface, q: int, r: int):
        """Draw a single hexagon with neon glow effect"""
        center_x, center_y = self.hex_to_pixel(q, r)
        
        # Skip if off screen
        if center_x < -HEX_SIZE or center_x > SCREEN_WIDTH + HEX_SIZE:
            return
        if center_y < -HEX_SIZE or center_y > SCREEN_HEIGHT + HEX_SIZE:
            return
        
        # Calculate hexagon vertices
        vertices = []
        for i in range(6):
            angle = math.pi / 3 * i
            x = center_x + HEX_SIZE * math.cos(angle)
            y = center_y + HEX_SIZE * math.sin(angle)
            vertices.append((x, y))
        
        # Get terrain color
        terrain = self.hex_map.get_tile(q, r)
        color = terrain.value
        
        # Draw filled hexagon
        pygame.draw.polygon(screen, color, vertices)
        
        # Draw neon border for glow effect
        border_color = tuple(min(255, c + 50) for c in color) if terrain != TerrainType.SNOW else (200, 200, 200)
        pygame.draw.polygon(screen, border_color, vertices, 2)
    
    def draw(self, screen: pygame.Surface):
        """Draw entire hex map"""
        for (q, r), terrain in self.hex_map.tiles.items():
            self.draw_hex(screen, q, r)
    
    def pan_camera(self, dx: int, dy: int):
        """Pan the camera"""
        self.camera_x -= dx
        self.camera_y -= dy

def draw_ui(screen: pygame.Surface):
    """Draw minimal UI with neon aesthetic"""
    # Draw title
    font = pygame.font.Font(None, 48)
    title = font.render("PANGAEA", True, (0, 255, 255))
    title_rect = title.get_rect(center=(SCREEN_WIDTH // 2, 40))
    
    # Add glow effect to title
    glow_surf = pygame.Surface((title_rect.width + 20, title_rect.height + 20))
    glow_surf.set_alpha(50)
    glow_surf.fill((0, 255, 255))
    screen.blit(glow_surf, (title_rect.x - 10, title_rect.y - 10))
    
    screen.blit(title, title_rect)
    
    # Draw controls in corner
    small_font = pygame.font.Font(None, 20)
    controls = [
        "ARROW KEYS: PAN",
        "ESC: EXIT"
    ]
    
    y_offset = SCREEN_HEIGHT - 60
    for control in controls:
        text = small_font.render(control, True, (0, 255, 100))
        screen.blit(text, (20, y_offset))
        y_offset += 25

def draw_grid_effect(screen: pygame.Surface):
    """Draw subtle grid lines for cyberpunk effect"""
    grid_color = (10, 10, 30)  # Very dark blue
    for x in range(0, SCREEN_WIDTH, 100):
        pygame.draw.line(screen, grid_color, (x, 0), (x, SCREEN_HEIGHT), 1)
    for y in range(0, SCREEN_HEIGHT, 100):
        pygame.draw.line(screen, grid_color, (0, y), (SCREEN_WIDTH, y), 1)

def main():
    """Main game loop"""
    screen = pygame.display.set_mode((SCREEN_WIDTH, SCREEN_HEIGHT))
    pygame.display.set_caption("Neon Pangaea")
    clock = pygame.time.Clock()
    
    # Create Pangaea map
    print("\n=== NEON PANGAEA ===")
    print("Generating cyberpunk supercontinent...")
    hex_map = PangaeaMap()
    renderer = HexMapRenderer(hex_map)
    
    print("\nMap generated!")
    print("Use arrow keys to explore the neon world")
    
    running = True
    keys_pressed = set()
    
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                else:
                    keys_pressed.add(event.key)
            elif event.type == pygame.KEYUP:
                keys_pressed.discard(event.key)
        
        # Handle continuous key presses for panning
        pan_speed = 8
        if pygame.K_LEFT in keys_pressed:
            renderer.pan_camera(-pan_speed, 0)
        if pygame.K_RIGHT in keys_pressed:
            renderer.pan_camera(pan_speed, 0)
        if pygame.K_UP in keys_pressed:
            renderer.pan_camera(0, -pan_speed)
        if pygame.K_DOWN in keys_pressed:
            renderer.pan_camera(0, pan_speed)
        
        # Clear screen with black
        screen.fill((0, 0, 0))
        
        # Draw subtle grid effect
        draw_grid_effect(screen)
        
        # Draw hex map
        renderer.draw(screen)
        
        # Draw minimal UI
        draw_ui(screen)
        
        # Update display
        pygame.display.flip()
        clock.tick(60)
    
    pygame.quit()

if __name__ == "__main__":
    main() 