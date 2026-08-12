#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QDebug>

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent), ui(new Ui::MainWindow) {
    ui->setupUi(this);
    this->onLineNumbersChanged(1);
    this->highlightCurrentLine();

    QFont monospaceFont("monospace");
    monospaceFont.setStyleHint(QFont::TypeWriter);
    monospaceFont.setPointSize(10);

    this->ui->lineNumbers->setFont(monospaceFont);
    this->ui->plainTextEdit->setFont(monospaceFont);
    this->ui->errorListWidget->setFont(monospaceFont);

    this->positionLabel = new QLabel("[1:1]", this);
    this->ui->statusbar->addPermanentWidget(this->positionLabel);

    this->formatSelection = new QComboBox(this);
    this->formatSelection->addItem("DNPM Datenmodell 2.1");
    this->formatSelection->addItem("SE:dip Datenmodell");
    this->formatSelection->addItem("GRZ Metadata 1.3.1");
    this->ui->toolBar->addWidget(this->formatSelection);

    this->severitySelection = new QComboBox(this);
    this->severitySelection->addItem("Show errors only");
    this->severitySelection->addItem("Show errors and warnings");
    this->severitySelection->addItem("Show all notices");
    this->ui->toolBar->addWidget(this->severitySelection);

    connect(ui->actionOpen, &QAction::triggered, this, &MainWindow::onOpenAction);
    connect(ui->actionSave, &QAction::triggered, this, &MainWindow::onSaveAction);
    connect(ui->actionSaveAs, &QAction::triggered, this, &MainWindow::onSaveAsAction);
    connect(ui->actionValidate, &QAction::triggered, this, &MainWindow::onValidateAction);
    connect(this->formatSelection, &QComboBox::currentTextChanged, [this](const QString &) {
        this->onValidateAction();
    });
    connect(this->severitySelection, &QComboBox::currentTextChanged, [this](const QString &) {
        this->onValidateAction();
    });
    connect(ui->actionAbout, &QAction::triggered, [this] {
        QMessageBox::about(this,
                           "About DNPM-Validator",
                           R"(
<html><body>
<p style="font-size: large; font-weight: bold;">DNPM-Validator</p>
<p>Application to validate and edit a data set in DNPM Datenmodell 2.1, SE:dip data model and GRZ Metadata format</p>
<p><a href="https://github.com/pcvolkmer/dnpm-validator">https://github.com/pcvolkmer/dnpm-validator</a></p>
</body></html>)"
        );
    });

    connect(ui->plainTextEdit, &QPlainTextEdit::blockCountChanged, this, &MainWindow::onLineNumbersChanged);
    connect(ui->plainTextEdit->verticalScrollBar(), &QScrollBar::valueChanged, [this](const int value) {
        ui->lineNumbers->verticalScrollBar()->setValue(value);
    });
    connect(ui->plainTextEdit, &QPlainTextEdit::cursorPositionChanged, [this] {
        this->highlightCurrentLine();
        this->positionLabel->setText(
            QString("[%1:%2]")
            .arg(ui->plainTextEdit->textCursor().blockNumber() + 1)
            .arg(ui->plainTextEdit->textCursor().columnNumber() + 1)
        );
    });

    connect(ui->errorListWidget, &QListWidget::itemClicked,
            [this](const QListWidgetItem *item) {
                const auto row = item->listWidget()->currentIndex().row();
                this->onErrorSelected(row);
            });
}

MainWindow::~MainWindow() {
    delete this->formatSelection;
    delete this->positionLabel;
    delete ui;
}

void MainWindow::onOpenAction() {
    this->filename = QFileDialog::getOpenFileName(
        this,
        "Open file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );

    if (!this->filename.isEmpty()) {
        if (QFile file(this->filename); file.open(QIODevice::ReadOnly | QIODevice::Text)) {
            const QByteArray content = file.readAll();

            ui->plainTextEdit->setPlainText(
                QString::fromUtf8(content)
            );

            this->onValidateAction();
        }
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onSaveAction() {
    if (!this->filename.isEmpty()) {
        if (QFile file(this->filename); file.open(QIODevice::WriteOnly | QIODevice::Text)) {
            this->onValidateAction();
            file.write(this->ui->plainTextEdit->toPlainText().toUtf8());
            file.close();
        }
        return;
    }
    this->onSaveAsAction();
}

void MainWindow::onSaveAsAction() {
    const auto selectedFilename = QFileDialog::getSaveFileName(
        this,
        "Save file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );
    if (!selectedFilename.isEmpty()) {
        this->filename = selectedFilename;
        this->onSaveAction();
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onValidateAction() {
    const auto json = ui->plainTextEdit->toPlainText();
    auto validationType = dnpmvalidation::ValidationType::Mtb;
    if (this->formatSelection->currentIndex() == 1) {
        validationType = dnpmvalidation::ValidationType::Rd;
    } else if (this->formatSelection->currentIndex() == 2) {
        validationType = dnpmvalidation::ValidationType::Grz;
    }
    auto reportSeverity = dnpmvalidation::Severity::Error;
    if (this->severitySelection->currentIndex() == 1) {
        reportSeverity = dnpmvalidation::Severity::Warning;
    } else if (this->severitySelection->currentIndex() == 2) {
        reportSeverity = dnpmvalidation::Severity::Information;
    }
    auto errors = dnpmvalidation::validate(rust::String(json.toStdString()), validationType, reportSeverity);

    this->errorList.clear();
    this->ui->errorListWidget->clear();

    if (errors.empty()) {
        this->markErrors();
        return;
    }

    QFont monospaceFont("monospace");
    monospaceFont.setStyleHint(QFont::TypeWriter);
    monospaceFont.setPointSize(10);

    for (auto error: errors) {
        this->errorList.push_back(error);

        auto *itemWidget = new QWidget();
        itemWidget->setFont(monospaceFont);

        QHBoxLayout layout(itemWidget);

        auto *lineLabel = new QLabel(QString("%1:").arg(error.start.line, 6));
        lineLabel->setFixedWidth(64);
        lineLabel->setFont(monospaceFont);
        layout.addWidget(lineLabel);

        auto *icon = new QLabel();
        if (error.severity == dnpmvalidation::Severity::Error) {
            icon->setPixmap(QIcon(":/resources/emblem-error.png").pixmap(12, 12));
        } else if (error.severity == dnpmvalidation::Severity::Warning) {
            icon->setPixmap(QIcon(":/resources/emblem-warning.png").pixmap(12, 12));
        } else if (error.severity == dnpmvalidation::Severity::Information) {
            icon->setPixmap(QIcon(":/resources/emblem-information.png").pixmap(12, 12));
        }

        layout.addWidget(icon);
        layout.addWidget(new QLabel(error.message.c_str()));

        if (!error.path.empty()) {
            auto *pathLabel = new QLabel(QString(" - %1").arg(error.path.c_str()));
            pathLabel->setStyleSheet("color: gray");
            pathLabel->setFont(monospaceFont);
            layout.addWidget(pathLabel);
        }
        layout.setContentsMargins(0, 2, 0, 2);

        auto *item = new QListWidgetItem();
        ui->errorListWidget->addItem(item);
        ui->errorListWidget->setItemWidget(item, itemWidget);
        item->setSizeHint(itemWidget->sizeHint());
    }
    ui->errorListWidget->updateGeometry();
    this->markErrors();
}

void MainWindow::onErrorSelected(const int index) const {
    const auto error = this->errorList.at(index);
    auto cursor = ui->plainTextEdit->textCursor();

    if (error.start.line == 0 || error.start.column == 0) {
        return;
    }

    ui->plainTextEdit->setFocus();

    cursor.clearSelection();
    cursor.setPosition(
        QTextCursor::Start
    );

    const auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.start.line - 1);
    cursor.setPosition(startTextBlock.position() + error.start.column - 1, QTextCursor::MoveAnchor);
    ui->plainTextEdit->setTextCursor(cursor);
    ui->plainTextEdit->ensureCursorVisible();

    this->highlightCurrentLine();
}

void MainWindow::onLineNumbersChanged(const int lineCount) const {
    QStringList lines;
    for (int i = 1; i <= lineCount; i++) {
        lines += QString::number(i).rightJustified(6, ' ');
    }
    this->ui->lineNumbers->setPlainText(lines.join('\n'));
}

void MainWindow::highlightCurrentLine() const {
    ui->lineNumbers->verticalScrollBar()->setValue(ui->plainTextEdit->verticalScrollBar()->value());

    const auto currentBlock = ui->plainTextEdit->textCursor().block();

    QTextCharFormat lineFormat;
    QBrush brush;
    brush.setStyle(Qt::SolidPattern);
    brush.setColor(QColor::fromRgba(qRgba(127, 127, 127, 16)));
    lineFormat.setBackground(brush);
    lineFormat.setProperty(QTextFormat::FullWidthSelection, true);

    if (const auto lineCursor = QTextCursor(
            ui->lineNumbers->document()->findBlockByLineNumber(currentBlock.firstLineNumber())); lineCursor.block().
        isValid()) {
        QList<QTextEdit::ExtraSelection> extraSelections;

        QTextEdit::ExtraSelection textSelection;
        textSelection.cursor = lineCursor;
        textSelection.format = lineFormat;

        extraSelections.append(textSelection);
        for (const auto &extra_selection: ui->lineNumbers->extraSelections()) {
            if (extra_selection.format.foreground() == Qt::red
                || extra_selection.format.foreground() == Qt::darkYellow) {
                extraSelections.append(extra_selection);
            }
        }

        ui->lineNumbers->setExtraSelections(extraSelections);
    }

    if (const auto textCursor = QTextCursor(currentBlock); textCursor.block().isValid()) {
        QList<QTextEdit::ExtraSelection> extraSelections;

        QTextEdit::ExtraSelection textSelection;
        textSelection.cursor = textCursor;
        textSelection.format = lineFormat;

        extraSelections.append(textSelection);
        for (const auto &extra_selection: ui->plainTextEdit->extraSelections()) {
            if (extra_selection.format.property(QTextFormat::FullWidthSelection) != true) {
                extraSelections.append(extra_selection);
            }
        }

        ui->plainTextEdit->setExtraSelections(extraSelections);
    }
}

void MainWindow::markErrors() {
    auto cursor = ui->plainTextEdit->textCursor();

    QList<QTextEdit::ExtraSelection> lineExtraSelections;
    QList<QTextEdit::ExtraSelection> textExtraSelections;

    QList<int> markedLines;

    auto errors = std::list(this->errorList.begin(), this->errorList.end());
    std::ranges::reverse(errors);

    for (const auto &error: errors) {
        if (error.start.line == 0 || error.start.column == 0 || error.severity ==
            dnpmvalidation::Severity::Information) {
            continue;
        }

        if (!markedLines.contains(error.start.line - 1)) {
            QTextEdit::ExtraSelection lineSelection;
            lineSelection.cursor =
                    QTextCursor(ui->lineNumbers->document()->findBlockByLineNumber(error.start.line - 1));
            QTextCharFormat lineFormat;
            if (error.severity == dnpmvalidation::Severity::Error) {
                lineFormat.setForeground(Qt::red);
                lineFormat.setBackground(QColor::fromRgba(qRgba(200, 0, 0, 24)));
            } else if (error.severity == dnpmvalidation::Severity::Warning) {
                lineFormat.setForeground(Qt::darkYellow);
                lineFormat.setBackground(QColor::fromRgba(qRgba(200, 200, 0, 24)));
            }
            lineFormat.setProperty(QTextFormat::FullWidthSelection, true);
            lineSelection.format = lineFormat;
            lineExtraSelections.append(lineSelection);
            markedLines.append(error.start.line - 1);
        }

        auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.start.line - 1);
        cursor.setPosition(startTextBlock.position() + error.start.column - 1, QTextCursor::MoveAnchor);

        auto endTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.end.line - 1);
        cursor.setPosition(endTextBlock.position() + error.end.column - 1, QTextCursor::KeepAnchor);

        QTextEdit::ExtraSelection textSelection;
        textSelection.cursor = cursor;
        QTextCharFormat textFormat;
        textFormat.setUnderlineStyle(QTextCharFormat::WaveUnderline);
        if (error.severity == dnpmvalidation::Severity::Error) {
            textFormat.setUnderlineColor(Qt::red);
        } else if (error.severity == dnpmvalidation::Severity::Warning) {
            textFormat.setUnderlineColor(Qt::darkYellow);
        }
        textSelection.format = textFormat;
        textExtraSelections.append(textSelection);
    }

    ui->lineNumbers->setExtraSelections(lineExtraSelections);
    ui->plainTextEdit->setExtraSelections(textExtraSelections);

    this->highlightCurrentLine();
}
