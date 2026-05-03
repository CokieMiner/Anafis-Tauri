export interface UncertaintyMetadata {
  upperBound: number;
  lowerBound?: number;
  upperType: 'abs' | 'rel';
  lowerType?: 'abs' | 'rel';
  upperSource: 'manual' | 'propagated';
  lowerSource?: 'manual' | 'propagated';
}

export interface ParsedUncertainty {
  nominal: number;
  metadata: UncertaintyMetadata;
}
