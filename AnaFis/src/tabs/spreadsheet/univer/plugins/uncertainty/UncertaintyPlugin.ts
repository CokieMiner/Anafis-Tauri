import {
  type Dependency,
  DependentOn,
  Injector,
  Plugin,
  setDependencies,
  UniverInstanceType,
} from '@univerjs/core';
import { UniverFormulaEnginePlugin } from '@univerjs/engine-formula';
import { UniverSheetsPlugin } from '@univerjs/sheets';
import { UniverSheetsNumfmtPlugin } from '@univerjs/sheets-numfmt';
import { UncertaintyEditController } from '@/tabs/spreadsheet/univer/plugins/uncertainty/controllers/UncertaintyEditController';
import { UncertaintyFormatController } from '@/tabs/spreadsheet/univer/plugins/uncertainty/controllers/UncertaintyFormatController';
import { UncertaintyInputController } from '@/tabs/spreadsheet/univer/plugins/uncertainty/controllers/UncertaintyInputController';
import { UncertaintyPropagationController } from '@/tabs/spreadsheet/univer/plugins/uncertainty/controllers/UncertaintyPropagationController';

@DependentOn(
  UniverSheetsPlugin,
  UniverFormulaEnginePlugin,
  UniverSheetsNumfmtPlugin
)
export class UncertaintyPlugin extends Plugin {
  static override pluginName = 'uncertainty-plugin';
  static override type = UniverInstanceType.UNIVER_SHEET;

  constructor(
    _config: undefined,
    override readonly _injector: Injector
  ) {
    super();
  }

  override onStarting(): void {
    const dependencies: Dependency[] = [
      [UncertaintyInputController],
      [UncertaintyFormatController],
      [UncertaintyEditController],
      [UncertaintyPropagationController],
    ];

    dependencies.forEach((d) => {
      this._injector.add(d);
    });
  }

  override onReady(): void {
    // Explicitly instantiate controllers to ensure their _init() methods run
    this._injector.get(UncertaintyInputController);
    this._injector.get(UncertaintyFormatController);
    this._injector.get(UncertaintyEditController);
    this._injector.get(UncertaintyPropagationController);
  }
}

setDependencies(UncertaintyPlugin, [Injector], 1);
