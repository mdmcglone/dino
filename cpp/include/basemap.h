#ifndef BASEMAP_H
#define BASEMAP_H

#include <map>
#include <utility>
#include <vector>
#include <cmath>
#include "terrain.h"

struct HexCoord {
    int q, r;
    
    bool operator<(const HexCoord& other) const {
        return q < other.q || (q == other.q && r < other.r);
    }
};

class BaseMap {
protected:
    int width, height;
    std::map<HexCoord, TerrainType> tiles;
    
public:
    BaseMap(int w = 30, int h = 25) : width(w), height(h) {
        generateMap();
    }
    
    virtual ~BaseMap() = default;
    
    virtual void generateMap() {
        // Default: all water
        for (int q = 0; q < width; ++q) {
            for (int r = 0; r < height; ++r) {
                tiles[{q, r}] = TerrainType::WATER;
            }
        }
    }
    
    void setTile(int q, int r, TerrainType terrain) {
        if (q >= 0 && q < width && r >= 0 && r < height) {
            tiles[{q, r}] = terrain;
        }
    }
    
    TerrainType getTile(int q, int r) const {
        auto it = tiles.find({q, r});
        return (it != tiles.end()) ? it->second : TerrainType::WATER;
    }
    
    float distanceFromPoint(int q, int r, int centerQ, int centerR) const {
        return (std::abs(q - centerQ) + std::abs(q + r - centerQ - centerR) + std::abs(r - centerR)) / 2.0f;
    }
    
    std::vector<HexCoord> getNeighbors(int q, int r) const {
        return {
            {q+1, r}, {q-1, r},
            {q, r+1}, {q, r-1},
            {q+1, r-1}, {q-1, r+1}
        };
    }
    
    const std::map<HexCoord, TerrainType>& getTiles() const { return tiles; }
    int getWidth() const { return width; }
    int getHeight() const { return height; }
};

#endif // BASEMAP_H 