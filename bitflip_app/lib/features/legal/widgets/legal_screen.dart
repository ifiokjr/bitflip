import 'dart:async';

import 'package:bitflip_app/app/theme/bitflip_theme.dart';
import 'package:bitflip_app/l10n/l10n.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:go_router/go_router.dart';
import 'package:url_launcher/url_launcher.dart';

enum LegalDocument { privacy, terms, support }

class LegalScreen extends HookWidget {
  const LegalScreen({required this.document, super.key});

  final LegalDocument document;

  @override
  Widget build(BuildContext context) {
    final content = _content(context, document);
    return Scaffold(
      body: SafeArea(
        child: SelectionArea(
          child: CustomScrollView(
            slivers: [
              SliverToBoxAdapter(
                child: Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 820),
                    child: Padding(
                      padding: EdgeInsets.fromLTRB(
                        MediaQuery.sizeOf(context).width < 700 ? 20 : 40,
                        24,
                        MediaQuery.sizeOf(context).width < 700 ? 20 : 40,
                        72,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          TextButton.icon(
                            onPressed: () => context.go('/'),
                            icon: const Icon(Icons.west_rounded),
                            label: Text(context.l10n.backToBitflip),
                          ),
                          const SizedBox(height: 56),
                          Semantics(
                            header: true,
                            child: Text(
                              content.title,
                              style: Theme.of(context).textTheme.displaySmall,
                            ),
                          ),
                          const SizedBox(height: 18),
                          Text(
                            context.l10n.legalEffectiveDate,
                            style: Theme.of(context).textTheme.labelLarge
                                ?.copyWith(color: BitflipColors.cyan),
                          ),
                          const SizedBox(height: 24),
                          ConstrainedBox(
                            constraints: const BoxConstraints(maxWidth: 680),
                            child: Text(
                              content.introduction,
                              style: Theme.of(context).textTheme.bodyLarge
                                  ?.copyWith(height: 1.6),
                            ),
                          ),
                          const SizedBox(height: 48),
                          for (final section in content.sections)
                            _DocumentSection(section: section),
                          if (document == LegalDocument.support) ...[
                            const SizedBox(height: 8),
                            FilledButton.icon(
                              onPressed: () => unawaited(_openSupport()),
                              icon: const Icon(Icons.open_in_new_rounded),
                              label: Text(context.l10n.openSupportRequest),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _DocumentSection extends HookWidget {
  const _DocumentSection({required this.section});

  final _LegalSection section;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 38),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Semantics(
              header: true,
              child: Text(
                section.title,
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            const SizedBox(height: 12),
            Text(section.body),
          ],
        ),
      ),
    );
  }
}

Future<void> _openSupport() async {
  final uri = Uri.https('github.com', '/ifiokjr/bitflip/issues/new');
  if (!await launchUrl(uri, mode: LaunchMode.externalApplication)) {
    throw StateError('Could not open Bitflip support.');
  }
}

_LegalContent _content(BuildContext context, LegalDocument document) {
  final l10n = context.l10n;
  return switch (document) {
    LegalDocument.privacy => _LegalContent(
      title: l10n.privacyTitle,
      introduction: l10n.privacyIntroduction,
      sections: [
        _LegalSection(l10n.privacyPublicTitle, l10n.privacyPublicBody),
        _LegalSection(l10n.privacyServiceTitle, l10n.privacyServiceBody),
        _LegalSection(l10n.privacyControlTitle, l10n.privacyControlBody),
        _LegalSection(l10n.privacyContactTitle, l10n.privacyContactBody),
      ],
    ),
    LegalDocument.terms => _LegalContent(
      title: l10n.termsTitle,
      introduction: l10n.termsIntroduction,
      sections: [
        _LegalSection(l10n.termsActionsTitle, l10n.termsActionsBody),
        _LegalSection(l10n.termsWalletTitle, l10n.termsWalletBody),
        _LegalSection(l10n.termsArtworkTitle, l10n.termsArtworkBody),
        _LegalSection(l10n.termsRiskTitle, l10n.termsRiskBody),
      ],
    ),
    LegalDocument.support => _LegalContent(
      title: l10n.supportTitle,
      introduction: l10n.supportIntroduction,
      sections: [
        _LegalSection(l10n.supportSafetyTitle, l10n.supportSafetyBody),
        _LegalSection(
          l10n.supportTransactionTitle,
          l10n.supportTransactionBody,
        ),
        _LegalSection(l10n.supportIncludeTitle, l10n.supportIncludeBody),
      ],
    ),
  };
}

final class _LegalContent {
  const _LegalContent({
    required this.title,
    required this.introduction,
    required this.sections,
  });

  final String title;
  final String introduction;
  final List<_LegalSection> sections;
}

final class _LegalSection {
  const _LegalSection(this.title, this.body);

  final String title;
  final String body;
}
