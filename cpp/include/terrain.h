#ifndef TERRAIN_H
#define TERRAIN_H

#include <SDL2/SDL.h>
#include <string>

enum class TerrainType {
    WATER,
    SHALLOW_WATER,
    GRASS,
    FOREST,
    DESERT,
    MOUNTAIN,
    SNOW,
    SAVANNA,
    JUNGLE
};

struct TerrainColor {
    SDL_Color color;
    SDL_Color glowColor;
    std::string name;
};

// Neon color palette
inline TerrainColor getTerrainColor(TerrainType type) {
    switch (type) {
        case TerrainType::WATER:
            return {{0, 180, 255, 255}, {0, 200, 255, 255}, "Water"};
        case TerrainType::SHALLOW_WATER:
            return {{0, 255, 255, 255}, {50, 255, 255, 255}, "Shallow Water"};
        case TerrainType::GRASS:
            return {{0, 255, 100, 255}, {50, 255, 150, 255}, "Grass"};
        case TerrainType::FOREST:
            return {{255, 0, 255, 255}, {255, 50, 255, 255}, "Forest"};
        case TerrainType::DESERT:
            return {{255, 180, 0, 255}, {255, 200, 50, 255}, "Desert"};
        case TerrainType::MOUNTAIN:
            return {{150, 0, 255, 255}, {200, 50, 255, 255}, "Mountain"};
        case TerrainType::SNOW:
            return {{255, 255, 255, 255}, {200, 200, 200, 255}, "Snow"};
        case TerrainType::SAVANNA:
            return {{255, 255, 0, 255}, {255, 255, 50, 255}, "Savanna"};
        case TerrainType::JUNGLE:
            return {{0, 255, 200, 255}, {50, 255, 250, 255}, "Jungle"};
        default:
            return {{0, 180, 255, 255}, {0, 200, 255, 255}, "Water"};
    }
}

#endif // TERRAIN_H 