#include "pangaea.h"
#include <algorithm>
#include <chrono>

PangaeaMap::PangaeaMap() : BaseMap(35, 30), rng(std::chrono::steady_clock::now().time_since_epoch().count()) {
}

void PangaeaMap::generateMap() {
    // First, fill everything with water
    for (int q = 0; q < width; ++q) {
        for (int r = 0; r < height; ++r) {
            tiles[{q, r}] = TerrainType::WATER;
        }
    }
    
    // Create the map features
    createPangaeaShape();
    addMountainRanges();
    addDeserts();
    addForestsAndJungles();
    addCoastalFeatures();
    addTerrainVariation();
}

void PangaeaMap::createPangaeaShape() {
    int centerQ = width / 2;
    int centerR = height / 2;
    
    // Define multiple overlapping regions to create irregular shape
    std::vector<Region> regions = {
        // Main body (central mass)
        {{centerQ, centerR}, 8.0f, 1.2f, 1.0f},
        // Northern extension (Laurasia)
        {{centerQ - 3, centerR - 5}, 6.0f, 1.5f, 0.8f},
        // Southern extension (Gondwana)
        {{centerQ + 2, centerR + 6}, 7.0f, 1.3f, 1.2f},
        // Western bulge
        {{centerQ - 7, centerR + 2}, 5.0f, 0.8f, 1.4f},
        // Eastern peninsula
        {{centerQ + 8, centerR - 2}, 4.0f, 1.6f, 0.7f}
    };
    
    std::uniform_real_distribution<float> noiseDist(0.0f, 1.5f);
    
    // Create landmass from overlapping elliptical regions
    for (const auto& region : regions) {
        for (int q = 0; q < width; ++q) {
            for (int r = 0; r < height; ++r) {
                // Calculate stretched distance
                float dx = (q - region.center.q) / region.stretchX;
                float dy = (r - region.center.r) / region.stretchY;
                float distance = std::sqrt(dx * dx + dy * dy);
                
                // Add some noise for irregular coastline
                float noise = noiseDist(rng);
                if (distance < region.radius + noise) {
                    tiles[{q, r}] = TerrainType::GRASS;
                }
            }
        }
    }
}

void PangaeaMap::addMountainRanges() {
    std::uniform_int_distribution<int> smallDist(-2, 2);
    std::uniform_int_distribution<int> tinyDist(-1, 1);
    
    // Central mountain range
    for (int i = 0; i < 15; ++i) {
        int q = width / 2 + smallDist(rng);
        int r = height / 2 - 8 + i + tinyDist(rng);
        placeMountainCluster(q, r, 2);
    }
    
    // Eastern range
    for (int i = 0; i < 10; ++i) {
        int q = width / 2 + 6 + tinyDist(rng);
        int r = height / 2 - 5 + i;
        placeMountainCluster(q, r, 1);
    }
}

void PangaeaMap::placeMountainCluster(int centerQ, int centerR, int size) {
    if (getTile(centerQ, centerR) != TerrainType::WATER) {
        setTile(centerQ, centerR, TerrainType::MOUNTAIN);
        
        if (size > 0) {
            std::uniform_real_distribution<float> chanceDist(0.0f, 1.0f);
            auto neighbors = getNeighbors(centerQ, centerR);
            
            for (const auto& n : neighbors) {
                if (chanceDist(rng) < 0.6f && getTile(n.q, n.r) != TerrainType::WATER) {
                    setTile(n.q, n.r, TerrainType::MOUNTAIN);
                }
            }
        }
    }
}

void PangaeaMap::addDeserts() {
    std::uniform_real_distribution<float> chanceDist(0.0f, 1.0f);
    
    for (int q = 0; q < width; ++q) {
        for (int r = 0; r < height; ++r) {
            if (getTile(q, r) == TerrainType::GRASS) {
                int waterDistance = distanceToWater(q, r);
                
                // Interior areas become desert
                if (waterDistance > 4) {
                    if (chanceDist(rng) < 0.7f) {
                        setTile(q, r, TerrainType::DESERT);
                    }
                } else if (waterDistance > 3 && chanceDist(rng) < 0.3f) {
                    setTile(q, r, TerrainType::SAVANNA);
                }
            }
        }
    }
}

void PangaeaMap::addForestsAndJungles() {
    std::uniform_real_distribution<float> chanceDist(0.0f, 1.0f);
    
    for (int q = 0; q < width; ++q) {
        for (int r = 0; r < height; ++r) {
            TerrainType terrain = getTile(q, r);
            if (terrain == TerrainType::GRASS || terrain == TerrainType::SAVANNA) {
                int waterDistance = distanceToWater(q, r);
                
                // Coastal areas get forests
                if (waterDistance <= 2) {
                    // Southern areas get jungle (tropical)
                    if (r > height * 0.6f) {
                        if (chanceDist(rng) < 0.5f) {
                            setTile(q, r, TerrainType::JUNGLE);
                        }
                    } else {
                        if (chanceDist(rng) < 0.4f) {
                            setTile(q, r, TerrainType::FOREST);
                        }
                    }
                }
            }
        }
    }
}

void PangaeaMap::addCoastalFeatures() {
    for (int q = 0; q < width; ++q) {
        for (int r = 0; r < height; ++r) {
            if (getTile(q, r) == TerrainType::WATER) {
                // Check if adjacent to land
                auto neighbors = getNeighbors(q, r);
                for (const auto& n : neighbors) {
                    if (n.q >= 0 && n.q < width && n.r >= 0 && n.r < height) {
                        TerrainType neighborTerrain = getTile(n.q, n.r);
                        if (neighborTerrain != TerrainType::WATER && 
                            neighborTerrain != TerrainType::SHALLOW_WATER) {
                            setTile(q, r, TerrainType::SHALLOW_WATER);
                            break;
                        }
                    }
                }
            }
        }
    }
}

void PangaeaMap::addTerrainVariation() {
    std::uniform_real_distribution<float> chanceDist(0.0f, 1.0f);
    
    for (int q = 0; q < width; ++q) {
        for (int r = 0; r < height; ++r) {
            TerrainType terrain = getTile(q, r);
            
            // Add snow to some mountain peaks
            if (terrain == TerrainType::MOUNTAIN) {
                auto neighbors = getNeighbors(q, r);
                int mountainNeighbors = 0;
                
                for (const auto& n : neighbors) {
                    if (n.q >= 0 && n.q < width && n.r >= 0 && n.r < height &&
                        getTile(n.q, n.r) == TerrainType::MOUNTAIN) {
                        mountainNeighbors++;
                    }
                }
                
                if (mountainNeighbors >= 4 && chanceDist(rng) < 0.3f) {
                    setTile(q, r, TerrainType::SNOW);
                }
            }
        }
    }
}

int PangaeaMap::distanceToWater(int q, int r) const {
    if (getTile(q, r) == TerrainType::WATER) {
        return 0;
    }
    
    std::set<HexCoord> visited;
    std::queue<std::tuple<int, int, int>> queue;
    queue.push({q, r, 0});
    visited.insert({q, r});
    
    while (!queue.empty()) {
        auto [cq, cr, dist] = queue.front();
        queue.pop();
        
        auto neighbors = getNeighbors(cq, cr);
        for (const auto& n : neighbors) {
            if (n.q >= 0 && n.q < width && n.r >= 0 && n.r < height &&
                visited.find(n) == visited.end()) {
                
                TerrainType nTerrain = getTile(n.q, n.r);
                if (nTerrain == TerrainType::WATER || nTerrain == TerrainType::SHALLOW_WATER) {
                    return dist + 1;
                }
                
                visited.insert(n);
                if (dist < 8) {  // Limit search distance
                    queue.push({n.q, n.r, dist + 1});
                }
            }
        }
    }
    
    return 10;  // Max distance
} 