#ifndef PANGAEA_H
#define PANGAEA_H

#include "basemap.h"
#include <random>
#include <queue>
#include <set>

struct Region {
    HexCoord center;
    float radius;
    float stretchX;
    float stretchY;
};

class PangaeaMap : public BaseMap {
private:
    std::mt19937 rng;
    
    void createPangaeaShape();
    void addMountainRanges();
    void addDeserts();
    void addForestsAndJungles();
    void addCoastalFeatures();
    void addTerrainVariation();
    void placeMountainCluster(int centerQ, int centerR, int size);
    int distanceToWater(int q, int r) const;
    
public:
    PangaeaMap();
    void generateMap() override;
};

#endif // PANGAEA_H 