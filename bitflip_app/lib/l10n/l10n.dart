import 'package:bitflip_app/l10n/generated/app_localizations.dart';
import 'package:flutter/widgets.dart';

extension LocalizationsBuildContext on BuildContext {
  AppLocalizations get l10n => AppLocalizations.of(this);
}
