#ifndef RENDERER_H
#define RENDERER_H

#include <SDL2/SDL.h>
#include <SDL2/SDL_ttf.h>
#include <memory>
#include <vector>
#include <cmath>
#include "basemap.h"

struct Point2D {
    float x, y;
};

class HexMapRenderer {
private:
    SDL_Renderer* renderer;
    std::shared_ptr<BaseMap> hexMap;
    TTF_Font* font;
    TTF_Font* smallFont;
    
    int cameraX = 0;
    int cameraY = 0;
    const int hexSize = 25;
    const int screenWidth;
    const int screenHeight;
    
    Point2D hexToPixel(int q, int r) const;
    HexCoord pixelToHex(int x, int y) const;
    HexCoord hexRound(float q, float r) const;
    void drawHex(int q, int r);
    void drawFilledPolygon(const std::vector<Point2D>& vertices, SDL_Color color);
    void drawPolygonOutline(const std::vector<Point2D>& vertices, SDL_Color color, int thickness);
    
public:
    HexMapRenderer(SDL_Renderer* r, std::shared_ptr<BaseMap> map, int w, int h);
    ~HexMapRenderer();
    
    void draw();
    void drawUI();
    void drawGridEffect();
    void panCamera(int dx, int dy);
};

#endif // RENDERER_H 