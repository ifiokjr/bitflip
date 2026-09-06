/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'dart:typed_data' as _idt;
import 'package:serverpod/serverpod.dart' as _is;

abstract class ColourCanvasState
    implements _is.TableRow<_is.UuidValue?>, _is.ProtocolSerialization {
  ColourCanvasState._({
    this.id,
    required this.gameIndex,
    required this.sectionIndex,
    required this.policyVersion,
    required this.highestRevision,
    required this.colours,
    required this.pixelRevisions,
    DateTime? updatedAt,
  }) : updatedAt = updatedAt ?? DateTime.now();

  factory ColourCanvasState({
    _is.UuidValue? id,
    required int gameIndex,
    required int sectionIndex,
    required int policyVersion,
    required int highestRevision,
    required _idt.ByteData colours,
    required _idt.ByteData pixelRevisions,
    DateTime? updatedAt,
  }) = _ColourCanvasStateImpl;

  factory ColourCanvasState.fromJson(Map<String, dynamic> jsonSerialization) {
    return ColourCanvasState(
      id: jsonSerialization['id'] == null
          ? null
          : _is.UuidValueJsonExtension.fromJson(jsonSerialization['id']),
      gameIndex: jsonSerialization['gameIndex'] as int,
      sectionIndex: jsonSerialization['sectionIndex'] as int,
      policyVersion: jsonSerialization['policyVersion'] as int,
      highestRevision: jsonSerialization['highestRevision'] as int,
      colours: _is.ByteDataJsonExtension.fromJson(jsonSerialization['colours']),
      pixelRevisions: _is.ByteDataJsonExtension.fromJson(
        jsonSerialization['pixelRevisions'],
      ),
      updatedAt: jsonSerialization['updatedAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(jsonSerialization['updatedAt']),
    );
  }

  static final t = ColourCanvasStateTable();

  static const db = ColourCanvasStateRepository._();

  @override
  _is.UuidValue? id;

  int gameIndex;

  int sectionIndex;

  int policyVersion;

  int highestRevision;

  _idt.ByteData colours;

  _idt.ByteData pixelRevisions;

  DateTime updatedAt;

  @override
  _is.Table<_is.UuidValue?> get table => t;

  /// Returns a shallow copy of this [ColourCanvasState]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  ColourCanvasState copyWith({
    _is.UuidValue? id,
    int? gameIndex,
    int? sectionIndex,
    int? policyVersion,
    int? highestRevision,
    _idt.ByteData? colours,
    _idt.ByteData? pixelRevisions,
    DateTime? updatedAt,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'ColourCanvasState',
      if (id != null) 'id': id?.toJson(),
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'policyVersion': policyVersion,
      'highestRevision': highestRevision,
      'colours': colours.toJson(),
      'pixelRevisions': pixelRevisions.toJson(),
      'updatedAt': updatedAt.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {};
  }

  static ColourCanvasStateInclude include() {
    return ColourCanvasStateInclude._();
  }

  static ColourCanvasStateIncludeList includeList({
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    ColourCanvasStateInclude? include,
  }) {
    return ColourCanvasStateIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _ColourCanvasStateImpl extends ColourCanvasState {
  _ColourCanvasStateImpl({
    _is.UuidValue? id,
    required int gameIndex,
    required int sectionIndex,
    required int policyVersion,
    required int highestRevision,
    required _idt.ByteData colours,
    required _idt.ByteData pixelRevisions,
    DateTime? updatedAt,
  }) : super._(
         id: id,
         gameIndex: gameIndex,
         sectionIndex: sectionIndex,
         policyVersion: policyVersion,
         highestRevision: highestRevision,
         colours: colours,
         pixelRevisions: pixelRevisions,
         updatedAt: updatedAt,
       );

  /// Returns a shallow copy of this [ColourCanvasState]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  ColourCanvasState copyWith({
    Object? id = _Undefined,
    int? gameIndex,
    int? sectionIndex,
    int? policyVersion,
    int? highestRevision,
    _idt.ByteData? colours,
    _idt.ByteData? pixelRevisions,
    DateTime? updatedAt,
  }) {
    return ColourCanvasState(
      id: id is _is.UuidValue? ? id : this.id,
      gameIndex: gameIndex ?? this.gameIndex,
      sectionIndex: sectionIndex ?? this.sectionIndex,
      policyVersion: policyVersion ?? this.policyVersion,
      highestRevision: highestRevision ?? this.highestRevision,
      colours: colours ?? this.colours.clone(),
      pixelRevisions: pixelRevisions ?? this.pixelRevisions.clone(),
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}

class ColourCanvasStateUpdateTable
    extends _is.UpdateTable<ColourCanvasStateTable> {
  ColourCanvasStateUpdateTable(super.table);

  _is.ColumnValue<int, int> gameIndex(int value) => _is.ColumnValue(
    table.gameIndex,
    value,
  );

  _is.ColumnValue<int, int> sectionIndex(int value) => _is.ColumnValue(
    table.sectionIndex,
    value,
  );

  _is.ColumnValue<int, int> policyVersion(int value) => _is.ColumnValue(
    table.policyVersion,
    value,
  );

  _is.ColumnValue<int, int> highestRevision(int value) => _is.ColumnValue(
    table.highestRevision,
    value,
  );

  _is.ColumnValue<_idt.ByteData, _idt.ByteData> colours(_idt.ByteData value) =>
      _is.ColumnValue(
        table.colours,
        value,
      );

  _is.ColumnValue<_idt.ByteData, _idt.ByteData> pixelRevisions(
    _idt.ByteData value,
  ) => _is.ColumnValue(
    table.pixelRevisions,
    value,
  );

  _is.ColumnValue<DateTime, DateTime> updatedAt(DateTime value) =>
      _is.ColumnValue(
        table.updatedAt,
        value,
      );
}

class ColourCanvasStateTable extends _is.Table<_is.UuidValue?> {
  ColourCanvasStateTable({super.tableRelation})
    : super(tableName: 'bitflip_colour_canvas_state') {
    updateTable = ColourCanvasStateUpdateTable(this);
    gameIndex = _is.ColumnInt(
      'gameIndex',
      this,
    );
    sectionIndex = _is.ColumnInt(
      'sectionIndex',
      this,
    );
    policyVersion = _is.ColumnInt(
      'policyVersion',
      this,
    );
    highestRevision = _is.ColumnInt(
      'highestRevision',
      this,
    );
    colours = _is.ColumnByteData(
      'colours',
      this,
    );
    pixelRevisions = _is.ColumnByteData(
      'pixelRevisions',
      this,
    );
    updatedAt = _is.ColumnDateTime(
      'updatedAt',
      this,
    );
  }

  late final ColourCanvasStateUpdateTable updateTable;

  late final _is.ColumnInt gameIndex;

  late final _is.ColumnInt sectionIndex;

  late final _is.ColumnInt policyVersion;

  late final _is.ColumnInt highestRevision;

  late final _is.ColumnByteData colours;

  late final _is.ColumnByteData pixelRevisions;

  late final _is.ColumnDateTime updatedAt;

  @override
  List<_is.Column> get columns => [
    id,
    gameIndex,
    sectionIndex,
    policyVersion,
    highestRevision,
    colours,
    pixelRevisions,
    updatedAt,
  ];
}

class ColourCanvasStateInclude extends _is.IncludeObject {
  ColourCanvasStateInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<_is.UuidValue?> get table => ColourCanvasState.t;
}

class ColourCanvasStateIncludeList extends _is.IncludeList {
  ColourCanvasStateIncludeList._({
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(ColourCanvasState.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<_is.UuidValue?> get table => ColourCanvasState.t;
}

class ColourCanvasStateRepository {
  const ColourCanvasStateRepository._();

  /// Returns a list of [ColourCanvasState]s matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order of the items use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// The maximum number of items can be set by [limit]. If no limit is set,
  /// all items matching the query will be returned.
  ///
  /// [offset] defines how many items to skip, after which [limit] (or all)
  /// items are read from the database.
  ///
  /// ```dart
  /// var persons = await Persons.db.find(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.firstName,
  ///   limit: 100,
  /// );
  /// ```
  Future<List<ColourCanvasState>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<ColourCanvasState>(
      where: where?.call(ColourCanvasState.t),
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [ColourCanvasState] matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// [offset] defines how many items to skip, after which the next one will be picked.
  ///
  /// ```dart
  /// var youngestPerson = await Persons.db.findFirstRow(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.age,
  /// );
  /// ```
  Future<ColourCanvasState?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? where,
    int? offset,
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<ColourCanvasState>(
      where: where?.call(ColourCanvasState.t),
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [ColourCanvasState] by its [id] or null if no such row exists.
  Future<ColourCanvasState?> findById(
    _is.DatabaseSession session,
    _is.UuidValue id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<ColourCanvasState>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [ColourCanvasState]s in the list and returns the inserted rows.
  ///
  /// The returned [ColourCanvasState]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// insert, none of the rows will be inserted.
  ///
  /// If [ignoreConflicts] is set to `true`, rows that conflict with existing
  /// rows are silently skipped, and only the successfully inserted rows are
  /// returned.
  ///
  /// If [noReturn] is set to `true`, the inserted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> insert(
    _is.DatabaseSession session,
    List<ColourCanvasState> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<ColourCanvasState>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [ColourCanvasState] and returns the inserted row.
  ///
  /// The returned [ColourCanvasState] will have its `id` field set.
  Future<ColourCanvasState> insertRow(
    _is.DatabaseSession session,
    ColourCanvasState row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<ColourCanvasState>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [ColourCanvasState]s in the list and returns the resulting rows.
  ///
  /// If a row conflicts on the given [conflictColumns], the existing row is
  /// updated with the new values. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies to rows matching the
  /// given expression. Conflicting rows that don't match are skipped and not
  /// returned, so the resulting list may be shorter than [rows].
  ///
  /// The returned [ColourCanvasState]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> upsert(
    _is.DatabaseSession session,
    List<ColourCanvasState> rows, {
    required _is.ColumnSelections<ColourCanvasStateTable> conflictColumns,
    _is.ColumnSelections<ColourCanvasStateTable>? updateColumns,
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<ColourCanvasState>(
      rows,
      conflictColumns: conflictColumns(ColourCanvasState.t),
      updateColumns: updateColumns?.call(ColourCanvasState.t),
      updateWhere: updateWhere?.call(ColourCanvasState.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [ColourCanvasState] and returns the resulting row.
  ///
  /// If the row conflicts on the given [conflictColumns], the existing row is
  /// updated. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies when the existing
  /// row matches the expression. Returns `null` if no row was affected — for
  /// example when [updateWhere] does not match the conflicting row.
  ///
  /// The returned [ColourCanvasState] will have its `id` field set.
  Future<ColourCanvasState?> upsertRow(
    _is.DatabaseSession session,
    ColourCanvasState row, {
    required _is.ColumnSelections<ColourCanvasStateTable> conflictColumns,
    _is.ColumnSelections<ColourCanvasStateTable>? updateColumns,
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<ColourCanvasState>(
      row,
      conflictColumns: conflictColumns(ColourCanvasState.t),
      updateColumns: updateColumns?.call(ColourCanvasState.t),
      updateWhere: updateWhere?.call(ColourCanvasState.t),
      transaction: transaction,
    );
  }

  /// Updates all [ColourCanvasState]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> update(
    _is.DatabaseSession session,
    List<ColourCanvasState> rows, {
    _is.ColumnSelections<ColourCanvasStateTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<ColourCanvasState>(
      rows,
      columns: columns?.call(ColourCanvasState.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [ColourCanvasState]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<ColourCanvasState> updateRow(
    _is.DatabaseSession session,
    ColourCanvasState row, {
    _is.ColumnSelections<ColourCanvasStateTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<ColourCanvasState>(
      row,
      columns: columns?.call(ColourCanvasState.t),
      transaction: transaction,
    );
  }

  /// Updates a single [ColourCanvasState] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<ColourCanvasState?> updateById(
    _is.DatabaseSession session,
    _is.UuidValue id, {
    required _is.ColumnValueListBuilder<ColourCanvasStateUpdateTable>
    columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<ColourCanvasState>(
      id,
      columnValues: columnValues(ColourCanvasState.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [ColourCanvasState]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<ColourCanvasStateUpdateTable>
    columnValues,
    required _is.WhereExpressionBuilder<ColourCanvasStateTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<ColourCanvasState>(
      columnValues: columnValues(ColourCanvasState.t.updateTable),
      where: where(ColourCanvasState.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [ColourCanvasState]s in the list and returns the deleted rows.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// This is an atomic operation, meaning that if one of the rows fail to
  /// be deleted, none of the rows will be deleted.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> delete(
    _is.DatabaseSession session,
    List<ColourCanvasState> rows, {
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<ColourCanvasState>(
      rows,
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [ColourCanvasState].
  Future<ColourCanvasState> deleteRow(
    _is.DatabaseSession session,
    ColourCanvasState row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<ColourCanvasState>(
      row,
      transaction: transaction,
    );
  }

  /// Deletes all rows matching the [where] expression.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourCanvasState>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ColourCanvasStateTable> where,
    _is.OrderByBuilder<ColourCanvasStateTable>? orderBy,
    _is.OrderByListBuilder<ColourCanvasStateTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<ColourCanvasState>(
      where: where(ColourCanvasState.t),
      orderBy: orderBy?.call(ColourCanvasState.t),
      orderByList: orderByList?.call(ColourCanvasState.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourCanvasStateTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<ColourCanvasState>(
      where: where?.call(ColourCanvasState.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [ColourCanvasState] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ColourCanvasStateTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<ColourCanvasState>(
      where: where(ColourCanvasState.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
