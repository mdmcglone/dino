#include "renderer.h"
#include <algorithm>
#include <string>

HexMapRenderer::HexMapRenderer(SDL_Renderer* r, std::shared_ptr<BaseMap> map, int w, int h)
    : renderer(r), hexMap(map), screenWidth(w), screenHeight(h) {
    
    // Initialize fonts
    font = TTF_OpenFont("/System/Library/Fonts/Helvetica.ttc", 48);
    smallFont = TTF_OpenFont("/System/Library/Fonts/Helvetica.ttc", 20);
    
    if (!font || !smallFont) {
        // Fallback to a common font path
        font = TTF_OpenFont("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", 48);
        smallFont = TTF_OpenFont("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", 20);
    }
}

HexMapRenderer::~HexMapRenderer() {
    if (font) TTF_CloseFont(font);
    if (smallFont) TTF_CloseFont(smallFont);
}

Point2D HexMapRenderer::hexToPixel(int q, int r) const {
    float x = hexSize * 3.0f/2.0f * q;
    float y = hexSize * std::sqrt(3.0f) * (r + q/2.0f);
    return {x + 50 - cameraX, y + 50 - cameraY};
}

HexCoord HexMapRenderer::pixelToHex(int x, int y) const {
    // Adjust for camera and offset
    float px = x - 50 + cameraX;
    float py = y - 50 + cameraY;
    
    // Convert to hex coordinates
    float q = (2.0f/3.0f * px) / hexSize;
    float r = (-1.0f/3.0f * px + std::sqrt(3.0f)/3.0f * py) / hexSize;
    
    return hexRound(q, r);
}

HexCoord HexMapRenderer::hexRound(float q, float r) const {
    float s = -q - r;
    int rq = std::round(q);
    int rr = std::round(r);
    int rs = std::round(s);
    
    float qDiff = std::abs(rq - q);
    float rDiff = std::abs(rr - r);
    float sDiff = std::abs(rs - s);
    
    if (qDiff > rDiff && qDiff > sDiff) {
        rq = -rr - rs;
    } else if (rDiff > sDiff) {
        rr = -rq - rs;
    }
    
    return {rq, rr};
}

void HexMapRenderer::drawFilledPolygon(const std::vector<Point2D>& vertices, SDL_Color color) {
    if (vertices.size() < 3) return;
    
    // Convert to SDL points
    std::vector<Sint16> vx, vy;
    for (const auto& v : vertices) {
        vx.push_back(static_cast<Sint16>(v.x));
        vy.push_back(static_cast<Sint16>(v.y));
    }
    
    // Draw filled polygon using triangulation
    SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);
    
    // Simple fan triangulation from first vertex
    for (size_t i = 1; i < vertices.size() - 1; ++i) {
        SDL_Vertex triangle[3] = {
            {{vertices[0].x, vertices[0].y}, {color.r, color.g, color.b, color.a}, {0, 0}},
            {{vertices[i].x, vertices[i].y}, {color.r, color.g, color.b, color.a}, {0, 0}},
            {{vertices[i+1].x, vertices[i+1].y}, {color.r, color.g, color.b, color.a}, {0, 0}}
        };
        SDL_RenderGeometry(renderer, nullptr, triangle, 3, nullptr, 0);
    }
}

void HexMapRenderer::drawPolygonOutline(const std::vector<Point2D>& vertices, SDL_Color color, int thickness) {
    SDL_SetRenderDrawColor(renderer, color.r, color.g, color.b, color.a);
    
    for (int t = 0; t < thickness; ++t) {
        for (size_t i = 0; i < vertices.size(); ++i) {
            size_t next = (i + 1) % vertices.size();
            SDL_RenderDrawLineF(renderer, 
                vertices[i].x + t, vertices[i].y + t,
                vertices[next].x + t, vertices[next].y + t);
        }
    }
}

void HexMapRenderer::drawHex(int q, int r) {
    Point2D center = hexToPixel(q, r);
    
    // Skip if off screen
    if (center.x < -hexSize || center.x > screenWidth + hexSize ||
        center.y < -hexSize || center.y > screenHeight + hexSize) {
        return;
    }
    
    // Calculate hexagon vertices
    std::vector<Point2D> vertices;
    for (int i = 0; i < 6; ++i) {
        float angle = M_PI / 3.0f * i;
        vertices.push_back({
            center.x + hexSize * std::cos(angle),
            center.y + hexSize * std::sin(angle)
        });
    }
    
    // Get terrain colors
    TerrainType terrain = hexMap->getTile(q, r);
    TerrainColor tc = getTerrainColor(terrain);
    
    // Draw filled hexagon
    drawFilledPolygon(vertices, tc.color);
    
    // Draw neon border for glow effect
    drawPolygonOutline(vertices, tc.glowColor, 2);
}

void HexMapRenderer::draw() {
    const auto& tiles = hexMap->getTiles();
    for (const auto& [coord, terrain] : tiles) {
        drawHex(coord.q, coord.r);
    }
}

void HexMapRenderer::drawUI() {
    if (!font || !smallFont) return;
    
    // Draw title with glow effect
    SDL_Color titleColor = {0, 255, 255, 255};
    SDL_Surface* titleSurface = TTF_RenderText_Blended(font, "PANGAEA", titleColor);
    if (titleSurface) {
        SDL_Texture* titleTexture = SDL_CreateTextureFromSurface(renderer, titleSurface);
        
        int titleX = (screenWidth - titleSurface->w) / 2;
        int titleY = 40;
        
        // Draw glow
        SDL_SetTextureAlphaMod(titleTexture, 50);
        SDL_Rect glowRect = {titleX - 10, titleY - 10, titleSurface->w + 20, titleSurface->h + 20};
        SDL_RenderCopy(renderer, titleTexture, nullptr, &glowRect);
        
        // Draw title
        SDL_SetTextureAlphaMod(titleTexture, 255);
        SDL_Rect titleRect = {titleX, titleY, titleSurface->w, titleSurface->h};
        SDL_RenderCopy(renderer, titleTexture, nullptr, &titleRect);
        
        SDL_DestroyTexture(titleTexture);
        SDL_FreeSurface(titleSurface);
    }
    
    // Draw controls
    SDL_Color controlColor = {0, 255, 100, 255};
    const char* controls[] = {"ARROW KEYS: PAN", "ESC: EXIT"};
    
    int yOffset = screenHeight - 60;
    for (const char* control : controls) {
        SDL_Surface* textSurface = TTF_RenderText_Blended(smallFont, control, controlColor);
        if (textSurface) {
            SDL_Texture* textTexture = SDL_CreateTextureFromSurface(renderer, textSurface);
            SDL_Rect textRect = {20, yOffset, textSurface->w, textSurface->h};
            SDL_RenderCopy(renderer, textTexture, nullptr, &textRect);
            SDL_DestroyTexture(textTexture);
            SDL_FreeSurface(textSurface);
            yOffset += 25;
        }
    }
}

void HexMapRenderer::drawGridEffect() {
    SDL_SetRenderDrawColor(renderer, 10, 10, 30, 255);
    
    // Vertical lines
    for (int x = 0; x < screenWidth; x += 100) {
        SDL_RenderDrawLine(renderer, x, 0, x, screenHeight);
    }
    
    // Horizontal lines
    for (int y = 0; y < screenHeight; y += 100) {
        SDL_RenderDrawLine(renderer, 0, y, screenWidth, y);
    }
}

void HexMapRenderer::panCamera(int dx, int dy) {
    cameraX -= dx;
    cameraY -= dy;
} 