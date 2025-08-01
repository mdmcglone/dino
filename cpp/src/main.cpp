#include <SDL2/SDL.h>
#include <SDL2/SDL_ttf.h>
#include <iostream>
#include <memory>
#include <set>
#include "pangaea.h"
#include "renderer.h"

const int SCREEN_WIDTH = 1400;
const int SCREEN_HEIGHT = 900;

int main(int argc, char* argv[]) {
    // Initialize SDL
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        std::cerr << "SDL initialization failed: " << SDL_GetError() << std::endl;
        return 1;
    }
    
    // Initialize SDL_ttf
    if (TTF_Init() < 0) {
        std::cerr << "SDL_ttf initialization failed: " << TTF_GetError() << std::endl;
        SDL_Quit();
        return 1;
    }
    
    // Create window
    SDL_Window* window = SDL_CreateWindow(
        "Neon Pangaea",
        SDL_WINDOWPOS_CENTERED,
        SDL_WINDOWPOS_CENTERED,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        SDL_WINDOW_SHOWN
    );
    
    if (!window) {
        std::cerr << "Window creation failed: " << SDL_GetError() << std::endl;
        TTF_Quit();
        SDL_Quit();
        return 1;
    }
    
    // Create renderer
    SDL_Renderer* sdlRenderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
    if (!sdlRenderer) {
        std::cerr << "Renderer creation failed: " << SDL_GetError() << std::endl;
        SDL_DestroyWindow(window);
        TTF_Quit();
        SDL_Quit();
        return 1;
    }
    
    // Enable blending for transparency
    SDL_SetRenderDrawBlendMode(sdlRenderer, SDL_BLENDMODE_BLEND);
    
    // Create Pangaea map
    std::cout << "\n=== NEON PANGAEA ===" << std::endl;
    std::cout << "Generating cyberpunk supercontinent..." << std::endl;
    
    auto hexMap = std::make_shared<PangaeaMap>();
    HexMapRenderer mapRenderer(sdlRenderer, hexMap, SCREEN_WIDTH, SCREEN_HEIGHT);
    
    std::cout << "\nMap generated!" << std::endl;
    std::cout << "Use arrow keys to explore the neon world" << std::endl;
    
    // Game loop
    bool running = true;
    SDL_Event event;
    std::set<SDL_Keycode> keysPressed;
    
    while (running) {
        // Handle events
        while (SDL_PollEvent(&event)) {
            switch (event.type) {
                case SDL_QUIT:
                    running = false;
                    break;
                    
                case SDL_KEYDOWN:
                    if (event.key.keysym.sym == SDLK_ESCAPE) {
                        running = false;
                    } else {
                        keysPressed.insert(event.key.keysym.sym);
                    }
                    break;
                    
                case SDL_KEYUP:
                    keysPressed.erase(event.key.keysym.sym);
                    break;
            }
        }
        
        // Handle continuous key presses for panning
        const int panSpeed = 8;
        if (keysPressed.count(SDLK_LEFT)) {
            mapRenderer.panCamera(-panSpeed, 0);
        }
        if (keysPressed.count(SDLK_RIGHT)) {
            mapRenderer.panCamera(panSpeed, 0);
        }
        if (keysPressed.count(SDLK_UP)) {
            mapRenderer.panCamera(0, -panSpeed);
        }
        if (keysPressed.count(SDLK_DOWN)) {
            mapRenderer.panCamera(0, panSpeed);
        }
        
        // Clear screen with black
        SDL_SetRenderDrawColor(sdlRenderer, 0, 0, 0, 255);
        SDL_RenderClear(sdlRenderer);
        
        // Draw subtle grid effect
        mapRenderer.drawGridEffect();
        
        // Draw hex map
        mapRenderer.draw();
        
        // Draw UI
        mapRenderer.drawUI();
        
        // Present
        SDL_RenderPresent(sdlRenderer);
    }
    
    // Cleanup
    SDL_DestroyRenderer(sdlRenderer);
    SDL_DestroyWindow(window);
    TTF_Quit();
    SDL_Quit();
    
    return 0;
} 